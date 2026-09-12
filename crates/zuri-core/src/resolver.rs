use crate::parser::{CallResolution, SymbolKind};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(crate) struct ResolutionSymbol {
    pub id: String,
    pub file: String,
    pub name: String,
    pub qualified_name: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolutionImport {
    pub file: String,
    pub module: String,
    pub imported_name: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolutionCall {
    pub id: i64,
    pub file: String,
    pub caller_symbol_id: Option<String>,
    pub target_text: String,
    pub simple_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolutionDecision {
    pub call_id: i64,
    pub target_symbol_id: Option<String>,
    pub resolution: CallResolution,
}

fn module_name_for_file(file: &str) -> String {
    let normalized = file.replace('\\', "/");
    let without_py = normalized.strip_suffix(".py").unwrap_or(&normalized);
    let mut parts: Vec<&str> = without_py.split('/').filter(|part| !part.is_empty()).collect();
    if parts.last().copied() == Some("__init__") {
        parts.pop();
    }
    parts.join(".")
}

fn package_name_for_file(file: &str) -> String {
    let normalized = file.replace('\\', "/");
    if normalized.ends_with("/__init__.py") || normalized == "__init__.py" {
        return module_name_for_file(file);
    }
    let mut parts: Vec<&str> = normalized.split('/').collect();
    parts.pop();
    parts.join(".")
}

fn absolute_module(current_file: &str, module: &str) -> String {
    if !module.starts_with('.') {
        return module.to_string();
    }
    let dots = module.chars().take_while(|ch| *ch == '.').count();
    let remainder = module[dots..].trim_start_matches('.');
    let package = package_name_for_file(current_file);
    let mut parts: Vec<&str> = package.split('.').filter(|part| !part.is_empty()).collect();
    let ascend = dots.saturating_sub(1);
    for _ in 0..ascend.min(parts.len()) {
        parts.pop();
    }
    if !remainder.is_empty() {
        parts.extend(remainder.split('.').filter(|part| !part.is_empty()));
    }
    parts.join(".")
}

fn parsed_imported_name(raw: &str, explicit_alias: Option<&str>) -> (String, Option<String>) {
    if let Some(alias) = explicit_alias {
        return (raw.trim().to_string(), Some(alias.to_string()));
    }
    if let Some((name, alias)) = raw.split_once(" as ") {
        return (name.trim().to_string(), Some(alias.trim().to_string()));
    }
    (raw.trim().to_string(), None)
}

fn unique_id<'a, I>(iter: I) -> Option<String>
where
    I: IntoIterator<Item = &'a ResolutionSymbol>,
{
    let mut ids = iter.into_iter().map(|symbol| symbol.id.clone());
    let first = ids.next()?;
    if ids.next().is_none() {
        Some(first)
    } else {
        None
    }
}

fn module_member(
    module: &str,
    member: &str,
    symbols: &[ResolutionSymbol],
    module_by_file: &HashMap<String, String>,
) -> Option<String> {
    unique_id(symbols.iter().filter(|symbol| {
        module_by_file.get(&symbol.file).is_some_and(|name| name == module)
            && symbol.qualified_name == member
    }))
}

fn local_qualified(
    file: &str,
    qualified: &str,
    symbols: &[ResolutionSymbol],
) -> Option<String> {
    unique_id(
        symbols
            .iter()
            .filter(|symbol| symbol.file == file && symbol.qualified_name == qualified),
    )
}

fn resolve_bare_local(
    call: &ResolutionCall,
    name: &str,
    caller: Option<&ResolutionSymbol>,
    symbols: &[ResolutionSymbol],
) -> Option<String> {
    let same_file: Vec<&ResolutionSymbol> = symbols
        .iter()
        .filter(|symbol| symbol.file == call.file && symbol.name == name)
        .collect();
    if same_file.is_empty() {
        return None;
    }

    if let Some(caller) = caller {
        let mut scope = caller.qualified_name.as_str();
        while let Some((parent, _)) = scope.rsplit_once('.') {
            let desired = format!("{parent}.{name}");
            if let Some(id) = unique_id(same_file.iter().copied().filter(|symbol| {
                symbol.qualified_name == desired && symbol.kind != SymbolKind::Method
            })) {
                return Some(id);
            }
            scope = parent;
        }
    }

    unique_id(
        same_file
            .into_iter()
            .filter(|symbol| !symbol.qualified_name.contains('.')),
    )
}

fn resolve_from_import(
    call: &ResolutionCall,
    name: &str,
    imports: &[ResolutionImport],
    symbols: &[ResolutionSymbol],
    module_by_file: &HashMap<String, String>,
) -> Option<String> {
    let mut candidates = Vec::new();
    for import in imports.iter().filter(|import| import.file == call.file) {
        let Some(raw_imported) = import.imported_name.as_deref() else {
            continue;
        };
        let (imported, alias) = parsed_imported_name(raw_imported, import.alias.as_deref());
        let bound = alias.as_deref().unwrap_or(&imported);
        if bound != name {
            continue;
        }
        let module = absolute_module(&call.file, &import.module);
        if let Some(id) = module_member(&module, &imported, symbols, module_by_file) {
            candidates.push(id);
        }
    }
    candidates.sort();
    candidates.dedup();
    (candidates.len() == 1).then(|| candidates.remove(0))
}

fn resolve_dotted_import(
    call: &ResolutionCall,
    target: &str,
    imports: &[ResolutionImport],
    symbols: &[ResolutionSymbol],
    module_by_file: &HashMap<String, String>,
) -> Option<String> {
    let mut candidates = Vec::new();
    for import in imports.iter().filter(|import| import.file == call.file) {
        match import.imported_name.as_deref() {
            None => {
                let module = absolute_module(&call.file, &import.module);
                let bound = import
                    .alias
                    .as_deref()
                    .unwrap_or_else(|| module.split('.').next().unwrap_or(&module));
                if let Some(rest) = target.strip_prefix(&format!("{bound}.")) {
                    let (target_module, member) = if import.alias.is_some() {
                        if let Some((submodule, member)) = rest.rsplit_once('.') {
                            (format!("{module}.{submodule}"), member)
                        } else {
                            (module.clone(), rest)
                        }
                    } else if let Some(rest) = target.strip_prefix(&format!("{module}.")) {
                        if let Some((submodule, member)) = rest.rsplit_once('.') {
                            (format!("{module}.{submodule}"), member)
                        } else {
                            (module.clone(), rest)
                        }
                    } else {
                        continue;
                    };
                    if let Some(id) = module_member(
                        &target_module,
                        member,
                        symbols,
                        module_by_file,
                    ) {
                        candidates.push(id);
                    }
                }
            }
            Some(raw_imported) => {
                let (imported, alias) =
                    parsed_imported_name(raw_imported, import.alias.as_deref());
                let bound = alias.as_deref().unwrap_or(&imported);
                let Some(rest) = target.strip_prefix(&format!("{bound}.")) else {
                    continue;
                };
                let module = absolute_module(&call.file, &import.module);

                let imported_module = if module.is_empty() {
                    imported.clone()
                } else {
                    format!("{module}.{imported}")
                };
                if let Some(id) = module_member(
                    &imported_module,
                    rest,
                    symbols,
                    module_by_file,
                ) {
                    candidates.push(id);
                }

                let class_member = format!("{imported}.{rest}");
                if let Some(id) = module_member(&module, &class_member, symbols, module_by_file) {
                    candidates.push(id);
                }
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    (candidates.len() == 1).then(|| candidates.remove(0))
}

pub(crate) fn resolve_calls(
    symbols: &[ResolutionSymbol],
    imports: &[ResolutionImport],
    calls: &[ResolutionCall],
) -> Vec<ResolutionDecision> {
    let by_id: HashMap<&str, &ResolutionSymbol> =
        symbols.iter().map(|symbol| (symbol.id.as_str(), symbol)).collect();
    let module_by_file: HashMap<String, String> = symbols
        .iter()
        .map(|symbol| (symbol.file.clone(), module_name_for_file(&symbol.file)))
        .collect();
    let known_modules: HashSet<String> = module_by_file.values().cloned().collect();

    calls
        .iter()
        .map(|call| {
            let caller = call
                .caller_symbol_id
                .as_deref()
                .and_then(|id| by_id.get(id).copied());
            let target = call.target_text.trim();

            if let Some((receiver, member)) = target.split_once('.') {
                if matches!(receiver, "self" | "cls") {
                    if let Some(caller) = caller.filter(|caller| caller.kind == SymbolKind::Method) {
                        if let Some((class_name, _)) = caller.qualified_name.rsplit_once('.') {
                            if let Some(id) = local_qualified(
                                &call.file,
                                &format!("{class_name}.{member}"),
                                symbols,
                            ) {
                                return ResolutionDecision {
                                    call_id: call.id,
                                    target_symbol_id: Some(id),
                                    resolution: CallResolution::Probable,
                                };
                            }
                        }
                    }
                }

                if let Some(id) = local_qualified(&call.file, target, symbols) {
                    return ResolutionDecision {
                        call_id: call.id,
                        target_symbol_id: Some(id),
                        resolution: CallResolution::Resolved,
                    };
                }

                if let Some(id) = resolve_dotted_import(
                    call,
                    target,
                    imports,
                    symbols,
                    &module_by_file,
                ) {
                    return ResolutionDecision {
                        call_id: call.id,
                        target_symbol_id: Some(id),
                        resolution: CallResolution::Resolved,
                    };
                }
            } else if let Some(name) = call.simple_name.as_deref() {
                if let Some(id) = resolve_bare_local(call, name, caller, symbols) {
                    return ResolutionDecision {
                        call_id: call.id,
                        target_symbol_id: Some(id),
                        resolution: CallResolution::Resolved,
                    };
                }
                if let Some(id) = resolve_from_import(
                    call,
                    name,
                    imports,
                    symbols,
                    &module_by_file,
                ) {
                    return ResolutionDecision {
                        call_id: call.id,
                        target_symbol_id: Some(id),
                        resolution: CallResolution::Resolved,
                    };
                }
            }

            let _ = &known_modules;
            ResolutionDecision {
                call_id: call.id,
                target_symbol_id: None,
                resolution: CallResolution::Unresolved,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol(id: &str, file: &str, name: &str, qualified: &str, kind: SymbolKind) -> ResolutionSymbol {
        ResolutionSymbol {
            id: id.into(),
            file: file.into(),
            name: name.into(),
            qualified_name: qualified.into(),
            kind,
        }
    }

    fn call(id: i64, file: &str, caller: Option<&str>, target: &str) -> ResolutionCall {
        ResolutionCall {
            id,
            file: file.into(),
            caller_symbol_id: caller.map(str::to_string),
            target_text: target.into(),
            simple_name: (!target.contains('.')).then(|| target.into()),
        }
    }

    #[test]
    fn lexical_scope_beats_same_name_elsewhere() {
        let symbols = vec![
            symbol("caller", "a.py", "inner", "outer.inner", SymbolKind::Function),
            symbol("wanted", "a.py", "helper", "outer.helper", SymbolKind::Function),
            symbol("other", "b.py", "helper", "helper", SymbolKind::Function),
        ];
        let decisions = resolve_calls(&symbols, &[], &[call(1, "a.py", Some("caller"), "helper")]);
        assert_eq!(decisions[0].target_symbol_id.as_deref(), Some("wanted"));
        assert_eq!(decisions[0].resolution, CallResolution::Resolved);
    }

    #[test]
    fn imported_alias_resolves_to_repository_symbol() {
        let symbols = vec![symbol(
            "target",
            "pkg/utils.py",
            "clean",
            "clean",
            SymbolKind::Function,
        )];
        let imports = vec![ResolutionImport {
            file: "app.py".into(),
            module: "pkg.utils".into(),
            imported_name: Some("clean as scrub".into()),
            alias: None,
        }];
        let decisions = resolve_calls(&symbols, &imports, &[call(1, "app.py", None, "scrub")]);
        assert_eq!(decisions[0].target_symbol_id.as_deref(), Some("target"));
        assert_eq!(decisions[0].resolution, CallResolution::Resolved);
    }

    #[test]
    fn self_method_is_probable_not_fact() {
        let symbols = vec![
            symbol("caller", "service.py", "run", "Worker.run", SymbolKind::Method),
            symbol("target", "service.py", "save", "Worker.save", SymbolKind::Method),
        ];
        let decisions = resolve_calls(&symbols, &[], &[call(1, "service.py", Some("caller"), "self.save")]);
        assert_eq!(decisions[0].target_symbol_id.as_deref(), Some("target"));
        assert_eq!(decisions[0].resolution, CallResolution::Probable);
    }

    #[test]
    fn unrelated_unique_global_is_not_falsely_resolved() {
        let symbols = vec![symbol("target", "other.py", "helper", "helper", SymbolKind::Function)];
        let decisions = resolve_calls(&symbols, &[], &[call(1, "app.py", None, "helper")]);
        assert_eq!(decisions[0].resolution, CallResolution::Unresolved);
    }
}
