use crate::{Confidence, EvidenceKind, Result, Severity, ZuriError};
use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    AsyncFunction,
    Class,
    Method,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolMetrics {
    pub branches: usize,
    pub loops: usize,
    pub returns: usize,
    pub exception_handlers: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub kind: SymbolKind,
    pub name: String,
    pub qualified_name: String,
    pub signature: String,
    pub parameters: Vec<String>,
    pub decorators: Vec<String>,
    pub location: SourceLocation,
    pub metrics: SymbolMetrics,
    pub concepts: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub module: String,
    pub imported_name: Option<String>,
    pub alias: Option<String>,
    pub line: usize,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CallResolution {
    Resolved,
    Probable,
    Unresolved,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSite {
    pub caller_symbol_id: Option<String>,
    pub target_text: String,
    pub simple_name: Option<String>,
    pub line: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSignal {
    pub rule_id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub confidence: Confidence,
    pub evidence: EvidenceKind,
    pub line: usize,
    pub remediation: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub calls: Vec<CallSite>,
    pub signals: Vec<ReviewSignal>,
    pub syntax_error_lines: Vec<usize>,
}

fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}
fn location(file: &str, node: Node<'_>) -> SourceLocation {
    let s = node.start_position();
    let e = node.end_position();
    SourceLocation {
        file: file.into(),
        start_line: s.row + 1,
        start_column: s.column + 1,
        end_line: e.row + 1,
        end_column: e.column + 1,
    }
}
fn stable_id(file: &str, kind: &str, qualified: &str) -> String {
    let raw = format!("{file}\0{kind}\0{qualified}");
    blake3::hash(raw.as_bytes()).to_hex()[..20].to_string()
}
fn parameters(node: Node<'_>, source: &str) -> Vec<String> {
    node.child_by_field_name("parameters")
        .map(|n| {
            text(n, source)
                .trim_matches(&['(', ')'][..])
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
struct SignalSpec<'a> {
    id: &'a str,
    title: &'a str,
    description: &'a str,
    severity: Severity,
    confidence: Confidence,
    evidence: EvidenceKind,
    remediation: Option<&'a str>,
}

fn signal(spec: SignalSpec<'_>, node: Node<'_>) -> ReviewSignal {
    ReviewSignal {
        rule_id: spec.id.into(),
        title: spec.title.into(),
        description: spec.description.into(),
        severity: spec.severity,
        confidence: spec.confidence,
        evidence: spec.evidence,
        line: node.start_position().row + 1,
        remediation: spec.remediation.map(str::to_string),
    }
}
fn collect_metrics(node: Node<'_>, m: &mut SymbolMetrics, c: &mut Vec<String>) {
    match node.kind() {
        "if_statement" => {
            m.branches += 1;
            c.push("conditional branching".into())
        }
        "for_statement" | "while_statement" => {
            m.loops += 1;
            c.push("iteration".into())
        }
        "return_statement" => {
            m.returns += 1;
            c.push("return values".into())
        }
        "except_clause" | "try_statement" => {
            m.exception_handlers += 1;
            c.push("exception handling".into())
        }
        "list_comprehension" => c.push("list comprehension".into()),
        "dictionary_comprehension" => c.push("dictionary comprehension".into()),
        "lambda" => c.push("lambda expression".into()),
        "call" => c.push("function calls".into()),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "function_definition" | "class_definition") {
            continue;
        }
        collect_metrics(child, m, c)
    }
}

fn decorators(node: Node<'_>, source: &str) -> Vec<String> {
    let Some(parent) = node.parent().filter(|p| p.kind() == "decorated_definition") else {
        return Vec::new();
    };
    let mut cursor = parent.walk();
    parent
        .children(&mut cursor)
        .filter(|child| child.kind() == "decorator")
        .map(|child| text(child, source).trim().to_string())
        .collect()
}

fn walk(
    node: Node<'_>,
    source: &str,
    file: &str,
    scope_names: &mut Vec<String>,
    scope_ids: &mut Vec<String>,
    out: &mut ParsedFile,
) {
    let kind = node.kind();
    let mut pushed = false;
    if matches!(kind, "function_definition" | "class_definition") {
        let name = node
            .child_by_field_name("name")
            .map(|n| text(n, source))
            .unwrap_or("<anonymous>");
        let parent_is_class = node.parent().is_some_and(|p| {
            p.kind() == "block" && p.parent().is_some_and(|pp| pp.kind() == "class_definition")
        });
        let is_async = kind == "function_definition"
            && text(node, source).trim_start().starts_with("async def ");
        let sk = if kind == "class_definition" {
            SymbolKind::Class
        } else if parent_is_class {
            SymbolKind::Method
        } else if is_async {
            SymbolKind::AsyncFunction
        } else {
            SymbolKind::Function
        };
        let mut parts = scope_names.clone();
        parts.push(name.to_string());
        let qualified = parts.join(".");
        let sid = stable_id(file, kind, &qualified);
        let params = if kind == "function_definition" {
            parameters(node, source)
        } else {
            vec![]
        };
        let signature = if kind == "function_definition" {
            format!("{name}({})", params.join(", "))
        } else {
            name.to_string()
        };
        let mut metrics = SymbolMetrics::default();
        let mut concepts = Vec::new();
        collect_metrics(node, &mut metrics, &mut concepts);
        concepts.sort();
        concepts.dedup();
        out.symbols.push(Symbol {
            id: sid.clone(),
            kind: sk,
            name: name.into(),
            qualified_name: qualified,
            signature,
            parameters: params,
            decorators: decorators(node, source),
            location: location(file, node),
            metrics,
            concepts,
        });
        scope_names.push(name.into());
        scope_ids.push(sid);
        pushed = true;
    }
    match kind {
        "import_statement" => {
            let raw = text(node, source).trim_start_matches("import ").trim();
            for part in raw.split(',') {
                let mut bits = part.split_whitespace();
                if let Some(module) = bits.next() {
                    let alias = if bits.next() == Some("as") {
                        bits.next().map(str::to_string)
                    } else {
                        None
                    };
                    out.imports.push(Import {
                        module: module.into(),
                        imported_name: None,
                        alias,
                        line: node.start_position().row + 1,
                    });
                }
            }
        }
        "import_from_statement" => {
            let raw = text(node, source);
            let body = raw.trim_start_matches("from ");
            let (module, imported) = body.split_once(" import ").unwrap_or(("", ""));
            for name in imported.trim_matches(&['(', ')'][..]).split(',') {
                let value = name.trim();
                if value.is_empty() {
                    continue;
                }
                let (imported_name, alias) = if let Some((name, alias)) = value.split_once(" as ") {
                    (name.trim().to_string(), Some(alias.trim().to_string()))
                } else {
                    (value.to_string(), None)
                };
                out.imports.push(Import {
                    module: module.trim().to_string(),
                    imported_name: Some(imported_name),
                    alias,
                    line: node.start_position().row + 1,
                });
            }
        }
        "call" => {
            if let Some(func) = node.child_by_field_name("function") {
                let target = text(func, source).trim().to_string();
                let simple = if func.kind() == "identifier" {
                    Some(target.clone())
                } else {
                    None
                };
                out.calls.push(CallSite {
                    caller_symbol_id: scope_ids.last().cloned(),
                    target_text: target,
                    simple_name: simple,
                    line: node.start_position().row + 1,
                });
            }
        }
        "except_clause" => {
            if text(node, source).trim_start().starts_with("except:") {
                out.signals.push(signal(
                    SignalSpec {
                        id: "PY-EXC-001",
                        title: "Bare except",
                        description:
                            "Bare `except:` catches exceptions that usually should remain visible.",
                        severity: Severity::Warning,
                        confidence: Confidence::High,
                        evidence: EvidenceKind::Fact,
                        remediation: Some(
                            "Catch a specific exception type or re-raise unexpected exceptions.",
                        ),
                    },
                    node,
                ));
            }
        }
        "parameters" => {
            let raw = text(node, source);
            if raw.contains("=[]")
                || raw.contains("= []")
                || raw.contains("={}")
                || raw.contains("= {}")
                || raw.contains("=set()")
                || raw.contains("= set()")
            {
                out.signals.push(signal(
                    SignalSpec {
                        id: "PY-ARG-001",
                        title: "Mutable default argument",
                        description: "A list, dict, or set used as a default parameter is created once at function definition time.",
                        severity: Severity::Warning,
                        confidence: Confidence::High,
                        evidence: EvidenceKind::Fact,
                        remediation: Some("Use `None` as the default and create the mutable value inside the function."),
                    },
                    node,
                ));
            }
        }
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                let n = text(left, source).trim();
                if matches!(
                    n,
                    "list"
                        | "dict"
                        | "set"
                        | "str"
                        | "int"
                        | "float"
                        | "bool"
                        | "type"
                        | "input"
                        | "print"
                ) {
                    out.signals.push(signal(
                        SignalSpec {
                            id: "PY-NAME-001",
                            title: "Built-in name shadowed",
                            description:
                                "This assignment shadows a commonly used Python built-in name.",
                            severity: Severity::Info,
                            confidence: Confidence::High,
                            evidence: EvidenceKind::Fact,
                            remediation: Some("Prefer a more specific variable name."),
                        },
                        node,
                    ));
                }
            }
        }
        "ERROR" => out.syntax_error_lines.push(node.start_position().row + 1),
        _ => {}
    }
    if kind == "call" {
        let raw = text(node, source).replace(' ', "");
        if raw.starts_with("eval(") || raw.starts_with("exec(") {
            let id = if raw.starts_with("eval(") {
                "PY-DYN-001"
            } else {
                "PY-DYN-002"
            };
            let title = if id.ends_with("001") {
                "Use of eval"
            } else {
                "Use of exec"
            };
            out.signals.push(signal(
                SignalSpec {
                    id,
                    title,
                    description: "Dynamic code execution can create security and maintenance risk when input is not fully trusted.",
                    severity: Severity::Warning,
                    confidence: Confidence::High,
                    evidence: EvidenceKind::Fact,
                    remediation: Some("Prefer explicit parsing or dispatch. If unavoidable, strictly constrain the input."),
                },
                node,
            ));
        }
        if raw.contains("subprocess.") && raw.contains("shell=True") {
            out.signals.push(signal(
                SignalSpec {
                    id: "PY-SEC-001",
                    title: "Subprocess with shell=True",
                    description: "`shell=True` can become command injection when command content is influenced by untrusted data.",
                    severity: Severity::Warning,
                    confidence: Confidence::Medium,
                    evidence: EvidenceKind::Inference,
                    remediation: Some("Prefer argument arrays with `shell=False` and validate any external input."),
                },
                node,
            ));
        }
    }
    if kind == "binary_operator" {
        let raw = text(node, source).replace(' ', "");
        if raw.ends_with("/0") || raw.ends_with("//0") || raw.ends_with("%0") {
            out.signals.push(signal(
                SignalSpec {
                    id: "PY-ARITH-001",
                    title: "Literal division by zero",
                    description: "This arithmetic expression has a literal zero divisor.",
                    severity: Severity::Error,
                    confidence: Confidence::High,
                    evidence: EvidenceKind::Fact,
                    remediation: Some("Use a non-zero divisor or guard the operation."),
                },
                node,
            ));
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, source, file, scope_names, scope_ids, out)
    }
    if pushed {
        scope_names.pop();
        scope_ids.pop();
    }
}

pub fn parse_python(file: &str, source: &str) -> Result<ParsedFile> {
    let mut parser = Parser::new();
    let language: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|e| ZuriError::Parse(e.to_string()))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| ZuriError::Parse(format!("failed to parse {file}")))?;
    let mut out = ParsedFile {
        symbols: vec![],
        imports: vec![],
        calls: vec![],
        signals: vec![],
        syntax_error_lines: vec![],
    };
    walk(
        tree.root_node(),
        source,
        file,
        &mut Vec::new(),
        &mut Vec::new(),
        &mut out,
    );
    out.syntax_error_lines.sort();
    out.syntax_error_lines.dedup();
    for line in out.syntax_error_lines.clone() {
        out.signals.push(ReviewSignal {
            rule_id: "PY-SYNTAX-001".into(),
            title: "Parser error".into(),
            description: "Tree-sitter encountered invalid or incomplete Python syntax.".into(),
            severity: Severity::Error,
            confidence: Confidence::High,
            evidence: EvidenceKind::Fact,
            line,
            remediation: Some("Correct the syntax around this location.".into()),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts() {
        let p = parse_python(
            "a.py",
            "def hello(name):\n    print(name)\n    return name\n",
        )
        .unwrap();
        assert_eq!(p.symbols.len(), 1);
        assert!(p.calls.iter().any(|c| c.target_text == "print"));
        assert_eq!(p.symbols[0].metrics.returns, 1);
    }
    #[test]
    fn mutable() {
        let p = parse_python("a.py", "def f(items=[]):\n    return items\n").unwrap();
        assert!(p.signals.iter().any(|s| s.rule_id == "PY-ARG-001"));
    }
}
