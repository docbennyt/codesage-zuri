from pathlib import Path
import re


def replace_once(path, old, new):
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


replace_once(
    "crates/zuri-core/src/lib.rs",
    "mod knowledge;\nmod model;\nmod parser;\nmod scanner;\nmod store;",
    "mod benchmark;\nmod knowledge;\nmod model;\nmod parser;\nmod resolver;\nmod scanner;\nmod store;",
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    "pub use knowledge::{KnowledgeEntry, KnowledgeHit, KnowledgePack};",
    "pub use benchmark::{benchmark_project, BenchmarkReport};\npub use knowledge::{KnowledgeEntry, KnowledgeHit, KnowledgePack};",
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    "    pub unresolved_calls: usize,\n    pub concepts_to_review: Vec<String>,",
    "    pub probable_calls: usize,\n    pub unresolved_calls: usize,\n    pub concepts_to_review: Vec<String>,",
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    "    let mut evidence = BTreeMap::new();\n    *evidence.entry(\"FACT\".into()).or_insert(0) += 1 + callers.len() + callees.len();\n    for finding in &findings {",
    "    let mut evidence = BTreeMap::new();\n    *evidence.entry(\"FACT\".into()).or_insert(0) += 1;\n    for edge in callers.iter().chain(callees.iter()) {\n        let label = match edge.resolution {\n            CallResolution::Probable => \"INFERENCE\",\n            CallResolution::Resolved | CallResolution::Unresolved => \"FACT\",\n        };\n        *evidence.entry(label.into()).or_insert(0) += 1;\n    }\n    for finding in &findings {",
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    '''    for edge in &explanation.callers {
        verified_facts.push(format!(
            "caller: {} at {}:{} ({:?})",
            edge.name, edge.file, edge.line, edge.resolution
        ));
    }
    for edge in &explanation.callees {
        verified_facts.push(format!(
            "callee: {} at {}:{} ({:?})",
            edge.name, edge.file, edge.line, edge.resolution
        ));
    }
''',
    '''    let mut edge_inferences = Vec::new();
    for edge in &explanation.callers {
        match edge.resolution {
            CallResolution::Resolved => verified_facts.push(format!(
                "caller: {} at {}:{} (resolved)",
                edge.name, edge.file, edge.line
            )),
            CallResolution::Probable => edge_inferences.push(format!(
                "probable caller relationship: {} at {}:{}",
                edge.name, edge.file, edge.line
            )),
            CallResolution::Unresolved => {}
        }
    }
    for edge in &explanation.callees {
        match edge.resolution {
            CallResolution::Resolved => verified_facts.push(format!(
                "callee: {} at {}:{} (resolved)",
                edge.name, edge.file, edge.line
            )),
            CallResolution::Probable => edge_inferences.push(format!(
                "probable callee relationship: {} at {}:{}",
                edge.name, edge.file, edge.line
            )),
            CallResolution::Unresolved => verified_facts.push(format!(
                "call expression: {} at {}:{} (target unresolved)",
                edge.name, edge.file, edge.line
            )),
        }
    }
''',
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    '''    let inferences = explanation
        .findings
        .iter()
        .filter(|f| f.evidence == EvidenceKind::Inference)
        .map(|f| {
            format!(
                "{} at {}:{}: {}",
                f.rule_id, f.location.file, f.location.start_line, f.description
            )
        })
        .collect();
''',
    '''    let mut inferences: Vec<String> = explanation
        .findings
        .iter()
        .filter(|f| f.evidence == EvidenceKind::Inference)
        .map(|f| {
            format!(
                "{} at {}:{}: {}",
                f.rule_id, f.location.file, f.location.start_line, f.description
            )
        })
        .collect();
    inferences.extend(edge_inferences);
''',
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    "        unresolved_calls: stats.calls_unresolved,\n        concepts_to_review,",
    "        probable_calls: stats.calls_probable,\n        unresolved_calls: stats.calls_unresolved,\n        concepts_to_review,",
)
replace_once(
    "crates/zuri-core/src/lib.rs",
    "    } else if warning_findings > 0 || stats.calls_unresolved > 0 {",
    "    } else if warning_findings > 0 || stats.calls_probable > 0 || stats.calls_unresolved > 0 {",
)

parser = Path("crates/zuri-core/src/parser.rs")
text = parser.read_text()
pattern = re.compile(r'''        "import_from_statement" => \{.*?        \}\n        "call" => \{''', re.S)
replacement = '''        "import_from_statement" => {
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
        "call" => {'''
text, count = pattern.subn(replacement, text, count=1)
if count != 1:
    raise SystemExit("failed to replace import_from_statement parser")
parser.write_text(text)

store = Path("crates/zuri-core/src/store.rs")
text = store.read_text()
old = "    parser::{CallResolution, ParsedFile, SourceLocation, Symbol, SymbolKind, SymbolMetrics},\n    CallEdgeView, Confidence, DiscoveredFile, EvidenceKind, Result, Severity, ZuriError,"
new = "    parser::{CallResolution, ParsedFile, SourceLocation, Symbol, SymbolKind, SymbolMetrics},\n    resolver::{resolve_calls as resolve_python_calls, ResolutionCall, ResolutionImport, ResolutionSymbol},\n    CallEdgeView, Confidence, DiscoveredFile, EvidenceKind, Result, Severity, ZuriError,"
if old not in text:
    raise SystemExit("store import marker missing")
text = text.replace(old, new, 1)
text = text.replace(
    "    pub calls_resolved: usize,\n    pub calls_unresolved: usize,",
    "    pub calls_resolved: usize,\n    pub calls_probable: usize,\n    pub calls_unresolved: usize,",
    1,
)
method_pattern = re.compile(r'''    pub fn resolve_calls\(&self\) -> Result<\(\)> \{.*?\n    \}\n    pub fn rebuild_findings''', re.S)
method_replacement = '''    pub fn resolve_calls(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE calls SET target_symbol_id=NULL,resolution='unresolved'",
            [],
        )?;

        let mut symbol_statement = self
            .conn
            .prepare("SELECT id,file_path,name,qualified_name,kind FROM symbols")?;
        let symbols: Vec<ResolutionSymbol> = symbol_statement
            .query_map([], |row| {
                Ok(ResolutionSymbol {
                    id: row.get(0)?,
                    file: row.get(1)?,
                    name: row.get(2)?,
                    qualified_name: row.get(3)?,
                    kind: kind_from_str(&row.get::<_, String>(4)?),
                })
            })?
            .collect::<rusqlite::Result<_>>()?;

        let mut import_statement = self
            .conn
            .prepare("SELECT file_path,module,imported_name,alias FROM imports")?;
        let imports: Vec<ResolutionImport> = import_statement
            .query_map([], |row| {
                Ok(ResolutionImport {
                    file: row.get(0)?,
                    module: row.get(1)?,
                    imported_name: row.get(2)?,
                    alias: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;

        let mut call_statement = self.conn.prepare(
            "SELECT id,file_path,caller_symbol_id,target_text,simple_name FROM calls ORDER BY id",
        )?;
        let calls: Vec<ResolutionCall> = call_statement
            .query_map([], |row| {
                Ok(ResolutionCall {
                    id: row.get(0)?,
                    file: row.get(1)?,
                    caller_symbol_id: row.get(2)?,
                    target_text: row.get(3)?,
                    simple_name: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;

        for decision in resolve_python_calls(&symbols, &imports, &calls) {
            let resolution = match decision.resolution {
                CallResolution::Resolved => "resolved",
                CallResolution::Probable => "probable",
                CallResolution::Unresolved => "unresolved",
            };
            self.conn.execute(
                "UPDATE calls SET target_symbol_id=?1,resolution=?2 WHERE id=?3",
                params![decision.target_symbol_id, resolution, decision.call_id],
            )?;
        }
        Ok(())
    }
    pub fn rebuild_findings'''
text, count = method_pattern.subn(method_replacement, text, count=1)
if count != 1:
    raise SystemExit("failed to replace resolve_calls")
text = text.replace(
    "            calls_resolved: count(\"SELECT COUNT(*) FROM calls WHERE resolution='resolved'\")?,\n            calls_unresolved: count(\"SELECT COUNT(*) FROM calls WHERE resolution!='resolved'\")?,",
    "            calls_resolved: count(\"SELECT COUNT(*) FROM calls WHERE resolution='resolved'\")?,\n            calls_probable: count(\"SELECT COUNT(*) FROM calls WHERE resolution='probable'\")?,\n            calls_unresolved: count(\"SELECT COUNT(*) FROM calls WHERE resolution='unresolved'\")?,",
    1,
)
store.write_text(text)

replace_once(
    "apps/zuri-cli/src/main.rs",
    '''    VibeCheck {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Doctor {''',
    '''    VibeCheck {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Benchmark {
        path: Option<PathBuf>,
        #[arg(long, default_value_t = 5)]
        rounds: usize,
        #[arg(long)]
        json: bool,
    },
    Doctor {''',
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    "Resolved calls: {}  unresolved calls: {}",
    "Resolved calls: {}  probable calls: {}  unresolved calls: {}",
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    "                    report.stats.calls_resolved,\n                    report.stats.calls_unresolved",
    "                    report.stats.calls_resolved,\n                    report.stats.calls_probable,\n                    report.stats.calls_unresolved",
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    "Imports: {}  resolved calls: {}  unresolved calls: {}",
    "Imports: {}  resolved calls: {}  probable calls: {}  unresolved calls: {}",
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    "                    stats.calls_resolved,\n                    stats.calls_unresolved,",
    "                    stats.calls_resolved,\n                    stats.calls_probable,\n                    stats.calls_unresolved,",
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    '"Zuri Vibe Check\\nReadiness: {}\\nErrors: {}  warnings: {}  unresolved calls: {}\\nBasis: {}",',
    '"Zuri Vibe Check\\nReadiness: {}\\nErrors: {}  warnings: {}  probable calls: {}  unresolved calls: {}\\nBasis: {}",',
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    "                    report.warning_findings,\n                    report.unresolved_calls,\n                    report.basis",
    "                    report.warning_findings,\n                    report.probable_calls,\n                    report.unresolved_calls,\n                    report.basis",
)
replace_once(
    "apps/zuri-cli/src/main.rs",
    '''        Command::Doctor { json } => {
            let report = zuri_core::doctor();''',
    '''        Command::Benchmark { path, rounds, json } => {
            let report = zuri_core::benchmark_project(&root(path)?, rounds)?;
            if json {
                print_json(&report)?;
            } else {
                println!(
                    "Zuri local benchmark\\nProject: {}\\nRounds: {}\\nCold index: {:.3} ms\\nIncremental median: {:.3} ms\\nSymbol lookup median: {}\\nKnowledge search median: {:.3} ms\\nIndex size: {} bytes\\nResolved/probable/unresolved calls: {}/{}/{}",
                    report.root.display(),
                    report.rounds,
                    report.cold_index_us as f64 / 1000.0,
                    report.incremental_median_us as f64 / 1000.0,
                    report
                        .symbol_lookup_median_us
                        .map(|value| format!("{:.3} ms", value as f64 / 1000.0))
                        .unwrap_or_else(|| "n/a".into()),
                    report.knowledge_search_median_us as f64 / 1000.0,
                    report.index_bytes,
                    report.stats.calls_resolved,
                    report.stats.calls_probable,
                    report.stats.calls_unresolved,
                );
                if let Some(rss) = report.peak_rss_kib {
                    println!("Process VmHWM: {rss} KiB");
                }
                for note in &report.notes {
                    println!("Note: {note}");
                }
            }
        }
        Command::Doctor { json } => {
            let report = zuri_core::doctor();''',
)

tests = Path("crates/zuri-core/tests/vertical_slice.rs")
text = tests.read_text()
marker = '''#[test]
fn knowledge_pack_searches_offline() {'''
addition = '''#[test]
fn import_alias_and_module_alias_resolve_without_global_guessing() {
    let root = temp_project(
        "from pkg.utils import clean as scrub\\nimport pkg.utils as u\\n\\ndef main(value):\\n    scrub(value)\\n    return u.clean(value)\\n",
    );
    fs::create_dir_all(root.join("pkg")).unwrap();
    fs::write(root.join("pkg/__init__.py"), "").unwrap();
    fs::write(root.join("pkg/utils.py"), "def clean(value):\\n    return value\\n").unwrap();
    zuri_core::index_project(&root, true).unwrap();

    let explanation = zuri_core::explain(&root, "main").unwrap();
    assert_eq!(explanation.callees.len(), 2);
    assert!(explanation
        .callees
        .iter()
        .all(|edge| edge.resolution == zuri_core::CallResolution::Resolved));
    assert!(explanation.callees.iter().all(|edge| edge.symbol_id.is_some()));
    cleanup(&root);
}

#[test]
fn self_method_is_probable_and_not_counted_as_unresolved() {
    let root = temp_project(
        "class Worker:\\n    def save(self):\\n        return 1\\n\\n    def run(self):\\n        return self.save()\\n",
    );
    zuri_core::index_project(&root, true).unwrap();
    let explanation = zuri_core::explain(&root, "Worker.run").unwrap();
    assert_eq!(explanation.callees.len(), 1);
    assert_eq!(
        explanation.callees[0].resolution,
        zuri_core::CallResolution::Probable
    );
    let stats = zuri_core::open_project(&root).unwrap().stats().unwrap();
    assert_eq!(stats.calls_probable, 1);
    assert_eq!(stats.calls_unresolved, 0);
    let bundle = zuri_core::evidence_bundle(&root, "Worker.run", "Explain the call").unwrap();
    assert!(bundle
        .inferences
        .iter()
        .any(|item| item.contains("probable callee relationship")));
    assert!(!bundle
        .verified_facts
        .iter()
        .any(|item| item.contains("callee: self.save")));
    cleanup(&root);
}

#[test]
fn unique_symbol_in_an_unimported_module_stays_unresolved() {
    let root = temp_project("def main():\\n    return helper()\\n");
    fs::write(root.join("other.py"), "def helper():\\n    return 1\\n").unwrap();
    zuri_core::index_project(&root, true).unwrap();
    let explanation = zuri_core::explain(&root, "main").unwrap();
    assert_eq!(explanation.callees.len(), 1);
    assert_eq!(
        explanation.callees[0].resolution,
        zuri_core::CallResolution::Unresolved
    );
    assert!(explanation.callees[0].symbol_id.is_none());
    cleanup(&root);
}

#[test]
fn benchmark_harness_runs_offline_on_small_project() {
    let root = temp_project("def hello(name):\\n    return name\\n");
    let report = zuri_core::benchmark_project(&root, 1).unwrap();
    assert_eq!(report.rounds, 1);
    assert!(!report.network_used);
    assert!(!report.model_used);
    assert_eq!(report.incremental_index_us.len(), 1);
    assert_eq!(report.knowledge_search_us.len(), 1);
    cleanup(&root);
}

#[test]
fn knowledge_pack_searches_offline() {'''
if marker not in text:
    raise SystemExit("test insertion marker missing")
tests.write_text(text.replace(marker, addition, 1))
