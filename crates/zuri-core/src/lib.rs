mod knowledge;
mod model;
mod parser;
mod scanner;
mod store;

use blake3::Hasher;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub use knowledge::{KnowledgeEntry, KnowledgeHit, KnowledgePack};
pub use model::{
    EvidenceBundle, LocalOpenAiProvider, ModelConfig, ModelProvider, ModelResponse, ModelStatus,
};
pub use parser::{CallResolution, ParsedFile, ReviewSignal, SourceLocation, Symbol, SymbolKind};
pub use scanner::{DiscoveredFile, ProjectScanner};
pub use store::{Finding, ProjectDb, ProjectStats};

#[derive(Debug, thiserror::Error)]
pub enum ZuriError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ZuriError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceKind {
    Fact,
    Documented,
    Inference,
    Model,
}

impl std::fmt::Display for EvidenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Fact => "FACT",
                Self::Documented => "DOCUMENTED",
                Self::Inference => "INFERENCE",
                Self::Model => "MODEL",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Low => "low",
                Self::Medium => "medium",
                Self::High => "high",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Info => "info",
                Self::Warning => "warning",
                Self::Error => "error",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub schema_version: u32,
    pub profile: String,
    pub languages: Vec<String>,
    pub model_required: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            profile: "eco".into(),
            languages: vec!["python".into()],
            model_required: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitReport {
    pub root: PathBuf,
    pub config_path: PathBuf,
    pub knowledge_pack_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexReport {
    pub root: PathBuf,
    pub discovered_files: usize,
    pub python_files: usize,
    pub parsed_files: usize,
    pub unchanged_files: usize,
    pub removed_files: usize,
    pub stats: ProjectStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEdgeView {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub resolution: CallResolution,
    pub symbol_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    pub symbol: Symbol,
    pub callers: Vec<CallEdgeView>,
    pub callees: Vec<CallEdgeView>,
    pub findings: Vec<Finding>,
    pub evidence_summary: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceNode {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub resolution: CallResolution,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub answer: String,
    pub evidence: EvidenceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeCheckReport {
    pub stats: ProjectStats,
    pub readiness: String,
    pub error_findings: usize,
    pub warning_findings: usize,
    pub unresolved_calls: usize,
    pub concepts_to_review: Vec<String>,
    pub priority_findings: Vec<Finding>,
    pub basis: String,
}

fn project_key(root: &Path) -> String {
    let mut h = Hasher::new();
    h.update(root.to_string_lossy().as_bytes());
    h.finalize().to_hex()[..16].to_string()
}

pub fn project_db_path(root: &Path) -> Result<PathBuf> {
    let root = fs::canonicalize(root)?;
    let dirs = ProjectDirs::from("dev", "codesage-zuri", "CodeSage Zuri")
        .ok_or_else(|| ZuriError::Config("cannot determine OS data directory".into()))?;
    let dir = dirs.cache_dir().join("projects").join(project_key(&root));
    fs::create_dir_all(&dir)?;
    Ok(dir.join("index.sqlite"))
}

pub fn open_project(root: &Path) -> Result<ProjectDb> {
    let root = fs::canonicalize(root)?;
    ProjectDb::open(&project_db_path(&root)?, &root)
}

pub fn init_project(root: &Path, force: bool) -> Result<InitReport> {
    let root = fs::canonicalize(root)?;
    let config_path = root.join(".zuri.toml");
    if config_path.exists() && !force {
        return Err(ZuriError::Config(format!(
            "{} already exists; use --force to replace it",
            config_path.display()
        )));
    }
    let body = toml::to_string_pretty(&ProjectConfig::default())
        .map_err(|e| ZuriError::Config(e.to_string()))?;
    fs::write(&config_path, body)?;
    let pack = KnowledgePack::ensure_builtin()?;
    Ok(InitReport {
        root,
        config_path,
        knowledge_pack_path: pack.path().to_path_buf(),
    })
}

pub fn index_project(root: &Path, rebuild: bool) -> Result<IndexReport> {
    let root = fs::canonicalize(root)?;
    let discovered = ProjectScanner::default().scan(&root)?;
    let mut db = ProjectDb::open(&project_db_path(&root)?, &root)?;
    if rebuild {
        db.clear_index()?;
    }
    let previous = db.file_hashes()?;
    let mut seen = HashSet::new();
    let mut parsed = 0;
    let mut unchanged = 0;
    let python_files = discovered
        .iter()
        .filter(|f| f.language.as_deref() == Some("python"))
        .count();
    let tx = db.begin()?;
    for file in &discovered {
        if file.language.as_deref() != Some("python") {
            continue;
        }
        let rel = file.relative_path.to_string_lossy().replace('\\', "/");
        seen.insert(rel.clone());
        if previous.get(&rel).is_some_and(|h| h == &file.hash) {
            unchanged += 1;
            continue;
        }
        let source = fs::read_to_string(&file.absolute_path)
            .map_err(|e| ZuriError::Parse(format!("{rel}: {e}")))?;
        let parsed_file = parser::parse_python(&rel, &source)?;
        store::replace_file_in_tx(&tx, file, &parsed_file)?;
        parsed += 1;
    }
    let removed = store::remove_missing_in_tx(&tx, &seen)?;
    tx.commit()?;
    db.resolve_calls()?;
    db.rebuild_findings()?;
    let stats = db.stats()?;
    Ok(IndexReport {
        root,
        discovered_files: discovered.len(),
        python_files,
        parsed_files: parsed,
        unchanged_files: unchanged,
        removed_files: removed,
        stats,
    })
}

pub fn explain(root: &Path, target: &str) -> Result<Explanation> {
    let db = open_project(root)?;
    let symbol = db
        .find_symbol(target)?
        .ok_or_else(|| ZuriError::NotFound(target.into()))?;
    let callers = db.callers(&symbol.id)?;
    let callees = db.callees(&symbol.id)?;
    let findings = db.findings_for_file(&symbol.location.file)?;
    let mut evidence = BTreeMap::new();
    *evidence.entry("FACT".into()).or_insert(0) += 1 + callers.len() + callees.len();
    for finding in &findings {
        *evidence.entry(finding.evidence.to_string()).or_insert(0) += 1;
    }
    Ok(Explanation {
        symbol,
        callers,
        callees,
        findings,
        evidence_summary: evidence,
    })
}

pub fn evidence_bundle(root: &Path, target: &str, question: &str) -> Result<EvidenceBundle> {
    let explanation = explain(root, target)?;
    let symbol = &explanation.symbol;
    let mut verified_facts = vec![
        format!(
            "{} is defined at {}:{}-{}",
            symbol.qualified_name,
            symbol.location.file,
            symbol.location.start_line,
            symbol.location.end_line
        ),
        format!("signature: {}", symbol.signature),
        format!(
            "structure: branches={}, loops={}, explicit_returns={}, exception_handlers={}",
            symbol.metrics.branches,
            symbol.metrics.loops,
            symbol.metrics.returns,
            symbol.metrics.exception_handlers
        ),
    ];
    for edge in &explanation.callers {
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
    for finding in &explanation.findings {
        if finding.evidence == EvidenceKind::Fact {
            verified_facts.push(format!(
                "finding {} at {}:{}: {}",
                finding.rule_id, finding.location.file, finding.location.start_line, finding.title
            ));
        }
    }

    let mut documented_knowledge = Vec::new();
    if let Ok(pack) = KnowledgePack::ensure_builtin() {
        for concept in &symbol.concepts {
            if let Ok(hits) = pack.search(concept, 1) {
                if let Some(hit) = hits.first() {
                    documented_knowledge.push(format!(
                        "{}: {} (source: {})",
                        hit.entry.title, hit.entry.body, hit.entry.source_url
                    ));
                }
            }
        }
    }

    let inferences = explanation
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

    let mut source_snippets = Vec::new();
    let source_path = fs::canonicalize(root)?.join(&symbol.location.file);
    if let Ok(source) = fs::read_to_string(source_path) {
        let lines: Vec<&str> = source.lines().collect();
        let start = symbol.location.start_line.saturating_sub(1);
        let end = symbol.location.end_line.min(start.saturating_add(40)).min(lines.len());
        if start < end {
            source_snippets.push(format!(
                "{}:{}-{}\n{}",
                symbol.location.file,
                start + 1,
                end,
                lines[start..end].join("\n")
            ));
        }
    }

    Ok(EvidenceBundle {
        question: question.into(),
        verified_facts,
        documented_knowledge,
        inferences,
        source_snippets,
    })
}

pub fn trace(root: &Path, target: &str, depth: usize) -> Result<Vec<TraceNode>> {
    let db = open_project(root)?;
    let start = db
        .find_symbol(target)?
        .ok_or_else(|| ZuriError::NotFound(target.into()))?;
    let mut result = Vec::new();
    let mut frontier = vec![(start.id, 0usize)];
    let mut visited = HashSet::new();
    while let Some((id, current_depth)) = frontier.pop() {
        if current_depth >= depth || !visited.insert(id.clone()) {
            continue;
        }
        for edge in db.callees(&id)? {
            result.push(TraceNode {
                name: edge.name.clone(),
                file: edge.file.clone(),
                line: edge.line,
                resolution: edge.resolution,
                depth: current_depth + 1,
            });
            if let Some(next) = edge.symbol_id {
                frontier.push((next, current_depth + 1));
            }
        }
    }
    Ok(result)
}

pub fn concepts(root: &Path, target: &str) -> Result<Vec<String>> {
    let db = open_project(root)?;
    Ok(db
        .find_symbol(target)?
        .ok_or_else(|| ZuriError::NotFound(target.into()))?
        .concepts)
}

pub fn quiz(root: &Path, target: &str) -> Result<Vec<QuizQuestion>> {
    let db = open_project(root)?;
    let symbol = db
        .find_symbol(target)?
        .ok_or_else(|| ZuriError::NotFound(target.into()))?;
    let mut questions = vec![
        QuizQuestion {
            question: format!("Where is `{}` defined?", symbol.qualified_name),
            answer: format!("{}:{}", symbol.location.file, symbol.location.start_line),
            evidence: EvidenceKind::Fact,
        },
        QuizQuestion {
            question: format!(
                "How many explicit return statements are inside `{}`?",
                symbol.qualified_name
            ),
            answer: symbol.metrics.returns.to_string(),
            evidence: EvidenceKind::Fact,
        },
    ];
    if let Some(call) = db.callees(&symbol.id)?.first() {
        questions.push(QuizQuestion {
            question: format!("Name one callable used by `{}`.", symbol.qualified_name),
            answer: call.name.clone(),
            evidence: EvidenceKind::Fact,
        });
    }
    Ok(questions)
}

pub fn quiz_topic(topic: &str) -> Result<Vec<QuizQuestion>> {
    let pack = KnowledgePack::ensure_builtin()?;
    let mut entries = pack.topic(topic)?;
    if entries.is_empty() {
        entries = pack
            .search(topic, 5)?
            .into_iter()
            .map(|hit| hit.entry)
            .collect();
    }
    if entries.is_empty() {
        return Err(ZuriError::NotFound(format!(
            "no installed knowledge entry for `{topic}`"
        )));
    }
    Ok(entries
        .into_iter()
        .take(5)
        .map(|entry| QuizQuestion {
            question: format!("Explain: {}", entry.title),
            answer: entry.body,
            evidence: EvidenceKind::Documented,
        })
        .collect())
}

pub fn vibe_check(root: &Path) -> Result<VibeCheckReport> {
    let db = open_project(root)?;
    let stats = db.stats()?;
    let findings = db.findings()?;
    let error_findings = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    let warning_findings = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();
    let mut concept_counts: BTreeMap<String, usize> = BTreeMap::new();
    for symbol in db.symbols(None)? {
        for concept in symbol.concepts {
            *concept_counts.entry(concept).or_insert(0) += 1;
        }
    }
    let mut ranked_concepts: Vec<(String, usize)> = concept_counts.into_iter().collect();
    ranked_concepts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let concepts_to_review = ranked_concepts
        .into_iter()
        .take(8)
        .map(|(concept, _)| concept)
        .collect();
    let priority_findings = findings
        .into_iter()
        .filter(|finding| finding.severity >= Severity::Warning)
        .take(10)
        .collect();
    let readiness = if error_findings > 0 {
        "stop-and-understand"
    } else if warning_findings > 0 || stats.calls_unresolved > 0 {
        "review-before-changing"
    } else {
        "ready-for-guided-change"
    };
    Ok(VibeCheckReport {
        stats: stats.clone(),
        readiness: readiness.into(),
        error_findings,
        warning_findings,
        unresolved_calls: stats.calls_unresolved,
        concepts_to_review,
        priority_findings,
        basis: "deterministic index, call graph, rules and concepts; no model used".into(),
    })
}

pub fn doctor() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert("platform".into(), std::env::consts::OS.into());
    out.insert("architecture".into(), std::env::consts::ARCH.into());
    out.insert(
        "cpu_threads".into(),
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .to_string(),
    );
    out.insert("network_required".into(), "no".into());
    out.insert("model_required".into(), "no".into());
    out.insert("profile".into(), "ECO-compatible deterministic core".into());
    out
}
