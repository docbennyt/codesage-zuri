use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use zuri_core::{EvidenceKind, Severity, SymbolKind};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn temp_project(source: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("zuri-test-{}-{stamp}-{id}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("a.py"), source).unwrap();
    root
}

fn cleanup(root: &Path) {
    if let Ok(db_path) = zuri_core::project_db_path(root) {
        if let Some(project_cache) = db_path.parent() {
            let _ = fs::remove_dir_all(project_cache);
        }
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn index_explain_trace_quiz_work_without_model() {
    let root = temp_project(
        "def helper(x):\n    return x\n\ndef main(value):\n    if value:\n        return helper(value)\n    return None\n",
    );
    let report = zuri_core::index_project(&root, true).unwrap();
    assert_eq!(report.python_files, 1);
    assert!(report.stats.symbols >= 2);

    let explanation = zuri_core::explain(&root, "main").unwrap();
    assert!(explanation.callees.iter().any(|call| call.name == "helper"));
    assert!(zuri_core::trace(&root, "main", 2)
        .unwrap()
        .iter()
        .any(|node| node.name == "helper"));
    assert!(!zuri_core::quiz(&root, "main").unwrap().is_empty());
    cleanup(&root);
}

#[test]
fn incremental_index_skips_unchanged_and_reparses_changes() {
    let root = temp_project("def first():\n    return 1\n");
    let first = zuri_core::index_project(&root, true).unwrap();
    assert_eq!(first.parsed_files, 1);

    let unchanged = zuri_core::index_project(&root, false).unwrap();
    assert_eq!(unchanged.parsed_files, 0);
    assert_eq!(unchanged.unchanged_files, 1);

    fs::write(
        root.join("a.py"),
        "def first():\n    return second()\n\ndef second():\n    return 2\n",
    )
    .unwrap();
    let changed = zuri_core::index_project(&root, false).unwrap();
    assert_eq!(changed.parsed_files, 1);
    assert_eq!(changed.unchanged_files, 0);
    assert!(changed.stats.symbols >= 2);
    assert!(changed.stats.calls_resolved >= 1);
    cleanup(&root);
}

#[test]
fn review_findings_preserve_fact_vs_inference() {
    let root = temp_project(
        "import subprocess\n\ndef risky(items=[]):\n    eval('1 + 1')\n    try:\n        value = 1 / 0\n    except:\n        subprocess.run('echo risky', shell=True)\n    return items\n",
    );
    zuri_core::index_project(&root, true).unwrap();
    let findings = zuri_core::open_project(&root).unwrap().findings().unwrap();

    for expected in ["PY-ARG-001", "PY-DYN-001", "PY-EXC-001", "PY-ARITH-001"] {
        assert!(findings.iter().any(|finding| {
            finding.rule_id == expected && finding.evidence == EvidenceKind::Fact
        }));
    }
    let shell = findings
        .iter()
        .find(|finding| finding.rule_id == "PY-SEC-001")
        .unwrap();
    assert_eq!(shell.evidence, EvidenceKind::Inference);
    assert_eq!(shell.severity, Severity::Warning);
    cleanup(&root);
}

#[test]
fn async_decorators_and_caller_identity_are_kept() {
    let root = temp_project(
        "def deco(fn):\n    return fn\n\ndef helper():\n    return 1\n\n@deco\nasync def worker():\n    return helper()\n\ndef entry():\n    return helper()\n",
    );
    zuri_core::index_project(&root, true).unwrap();

    let worker = zuri_core::explain(&root, "worker").unwrap();
    assert_eq!(worker.symbol.kind, SymbolKind::AsyncFunction);
    assert!(worker
        .symbol
        .decorators
        .iter()
        .any(|value| value.contains("@deco")));

    let helper = zuri_core::explain(&root, "helper").unwrap();
    let caller_names: Vec<&str> = helper
        .callers
        .iter()
        .map(|call| call.name.as_str())
        .collect();
    assert!(caller_names.contains(&"worker"));
    assert!(caller_names.contains(&"entry"));
    cleanup(&root);
}

#[test]
fn init_and_topic_quiz_work_offline() {
    let root = temp_project("def hello():\n    return 'hi'\n");
    let report = zuri_core::init_project(&root, false).unwrap();
    assert!(report.config_path.exists());
    assert!(report.knowledge_pack_path.exists());
    assert!(zuri_core::init_project(&root, false).is_err());
    assert!(!zuri_core::quiz_topic("mutable-defaults")
        .unwrap()
        .is_empty());
    cleanup(&root);
}

#[test]
fn vibe_check_is_deterministic_and_prioritizes_blocking_errors() {
    let root = temp_project("def warning(items=[]):\n    return items\n");
    fs::write(
        root.join("z_error.py"),
        "def broken():\n    return 10 / 0\n",
    )
    .unwrap();
    zuri_core::index_project(&root, true).unwrap();
    let report = zuri_core::vibe_check(&root).unwrap();
    assert_eq!(report.readiness, "stop-and-understand");
    assert!(report.error_findings >= 1);
    assert_eq!(
        report
            .priority_findings
            .first()
            .map(|finding| finding.severity),
        Some(Severity::Error)
    );
    assert_eq!(
        report.basis,
        "deterministic index, call graph, rules and concepts; no model used"
    );
    cleanup(&root);
}

#[test]
fn import_alias_and_module_alias_resolve_without_global_guessing() {
    let root = temp_project(
        "from pkg.utils import clean as scrub\nimport pkg.utils as u\n\ndef main(value):\n    scrub(value)\n    return u.clean(value)\n",
    );
    fs::create_dir_all(root.join("pkg")).unwrap();
    fs::write(root.join("pkg/__init__.py"), "").unwrap();
    fs::write(
        root.join("pkg/utils.py"),
        "def clean(value):\n    return value\n",
    )
    .unwrap();
    zuri_core::index_project(&root, true).unwrap();

    let explanation = zuri_core::explain(&root, "main").unwrap();
    assert_eq!(explanation.callees.len(), 2);
    assert!(explanation
        .callees
        .iter()
        .all(|edge| edge.resolution == zuri_core::CallResolution::Resolved));
    assert!(explanation
        .callees
        .iter()
        .all(|edge| edge.symbol_id.is_some()));
    cleanup(&root);
}

#[test]
fn self_method_is_probable_and_not_counted_as_unresolved() {
    let root = temp_project(
        "class Worker:\n    def save(self):\n        return 1\n\n    def run(self):\n        return self.save()\n",
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
    let root = temp_project("def main():\n    return helper()\n");
    fs::write(root.join("other.py"), "def helper():\n    return 1\n").unwrap();
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
    let root = temp_project("def hello(name):\n    return name\n");
    let report = zuri_core::benchmark_project(&root, 1).unwrap();
    assert_eq!(report.rounds, 1);
    assert!(!report.network_used);
    assert!(!report.model_used);
    assert_eq!(report.incremental_index_us.len(), 1);
    assert_eq!(report.knowledge_search_us.len(), 1);
    cleanup(&root);
}

#[test]
fn knowledge_pack_searches_offline() {
    let pack = zuri_core::KnowledgePack::ensure_builtin().unwrap();
    assert!(!pack.search("mutable default", 5).unwrap().is_empty());
}
