mod tui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json as json_value;
use std::path::{Path, PathBuf};
use zuri_core::{KnowledgePack, LocalOpenAiProvider, ModelConfig, ModelProvider, Severity};

#[derive(Parser)]
#[command(
    name = "zuri",
    version,
    about = "CodeSage Zuri — Offline Code Literacy Engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Init {
        path: Option<PathBuf>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    Index {
        path: Option<PathBuf>,
        #[arg(long)]
        rebuild: bool,
        #[arg(long)]
        json: bool,
    },
    Status {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Map {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Symbols {
        query: Option<String>,
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Explain {
        target: String,
        path: Option<PathBuf>,
        #[arg(long)]
        model: bool,
        #[arg(long)]
        json: bool,
    },
    Trace {
        target: String,
        path: Option<PathBuf>,
        #[arg(long, default_value_t = 2)]
        depth: usize,
        #[arg(long)]
        json: bool,
    },
    Review {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        severity: Option<SeverityArg>,
    },
    Why {
        id: String,
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Concepts {
        target: String,
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Search {
        query: String,
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        json: bool,
    },
    Learn {
        topic: String,
        #[arg(long)]
        json: bool,
    },
    Quiz {
        target: Option<String>,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    VibeCheck {
        path: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Model {
        #[command(subcommand)]
        command: ModelCommand,
    },
    Tui {
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ModelCommand {
    Status,
    Configure {
        #[arg(long)]
        endpoint: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = true)]
        enabled: bool,
    },
    Test,
}

#[derive(Clone, Copy, ValueEnum)]
enum SeverityArg {
    Info,
    Warning,
    Error,
}

impl From<SeverityArg> for Severity {
    fn from(value: SeverityArg) -> Self {
        match value {
            SeverityArg::Info => Self::Info,
            SeverityArg::Warning => Self::Warning,
            SeverityArg::Error => Self::Error,
        }
    }
}

fn root(path: Option<PathBuf>) -> Result<PathBuf> {
    Ok(std::fs::canonicalize(
        path.unwrap_or(std::env::current_dir()?),
    )?)
}

fn db(root: &Path) -> Result<zuri_core::ProjectDb> {
    let db = zuri_core::open_project(root)?;
    if db.stats()?.files == 0 {
        anyhow::bail!("project has no index yet; run `zuri index .` first")
    }
    Ok(db)
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_explanation(explanation: &zuri_core::Explanation) {
    let symbol = &explanation.symbol;
    println!(
        "{}\n{}:{}-{}\n\nSignature\n  {}\n\nStructure\n  branches: {}\n  loops: {}\n  explicit returns: {}\n  exception handlers: {}",
        symbol.qualified_name,
        symbol.location.file,
        symbol.location.start_line,
        symbol.location.end_line,
        symbol.signature,
        symbol.metrics.branches,
        symbol.metrics.loops,
        symbol.metrics.returns,
        symbol.metrics.exception_handlers
    );
    if !explanation.callers.is_empty() {
        println!("\nCalled by");
        for edge in &explanation.callers {
            println!(
                "  {} [{:?}] {}:{}",
                edge.name, edge.resolution, edge.file, edge.line
            );
        }
    }
    if !explanation.callees.is_empty() {
        println!("\nCalls");
        for edge in &explanation.callees {
            println!(
                "  {} [{:?}] {}:{}",
                edge.name, edge.resolution, edge.file, edge.line
            );
        }
    }
    if !symbol.concepts.is_empty() {
        println!("\nConcepts\n  {}", symbol.concepts.join("\n  "));
    }
    println!("\nEvidence");
    for (kind, count) in &explanation.evidence_summary {
        println!("  {kind}: {count}");
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Tui { path: None }) {
        Command::Init { path, force, json } => {
            let report = zuri_core::init_project(&root(path)?, force)?;
            if json {
                print_json(&report)?;
            } else {
                println!(
                    "Initialized CodeSage Zuri\nProject: {}\nConfig: {}\nPython knowledge pack: {}\nNetwork used: no\nModel used: no",
                    report.root.display(),
                    report.config_path.display(),
                    report.knowledge_pack_path.display()
                );
            }
        }
        Command::Index {
            path,
            rebuild,
            json,
        } => {
            let report = zuri_core::index_project(&root(path)?, rebuild)?;
            if json {
                print_json(&report)?;
            } else {
                println!(
                    "Indexed {}\nPython files: {}\nParsed: {}  unchanged: {}  removed: {}\nSymbols: {}  findings: {}\nResolved calls: {}  unresolved calls: {}\nNetwork used: no\nModel used: no",
                    report.root.display(),
                    report.python_files,
                    report.parsed_files,
                    report.unchanged_files,
                    report.removed_files,
                    report.stats.symbols,
                    report.stats.findings,
                    report.stats.calls_resolved,
                    report.stats.calls_unresolved
                );
            }
        }
        Command::Status { path, json } | Command::Map { path, json } => {
            let root = root(path)?;
            let db = db(&root)?;
            let stats = db.stats()?;
            if json {
                print_json(&stats)?;
            } else {
                println!(
                    "Project: {}\nFiles: {}  symbols: {}  functions/methods: {}  classes: {}\nImports: {}  resolved calls: {}  unresolved calls: {}\nFindings: {}\nIndex: {}",
                    root.display(),
                    stats.files,
                    stats.symbols,
                    stats.functions,
                    stats.classes,
                    stats.imports,
                    stats.calls_resolved,
                    stats.calls_unresolved,
                    stats.findings,
                    db.path().display()
                );
            }
        }
        Command::Symbols { query, path, json } => {
            let symbols = db(&root(path)?)?.symbols(query.as_deref())?;
            if json {
                print_json(&symbols)?;
            } else {
                for symbol in symbols {
                    println!(
                        "{}\t{}\t{}:{}",
                        symbol.qualified_name,
                        symbol.signature,
                        symbol.location.file,
                        symbol.location.start_line
                    );
                }
            }
        }
        Command::Explain {
            target,
            path,
            model,
            json,
        } => {
            let root = root(path)?;
            let explanation = zuri_core::explain(&root, &target)?;
            let model_response = if model {
                let bundle = zuri_core::evidence_bundle(
                    &root,
                    &target,
                    &format!("Explain `{target}` using only the supplied CodeSage evidence."),
                )?;
                let provider = LocalOpenAiProvider::new(ModelConfig::load()?)?;
                Some(provider.complete(&bundle)?)
            } else {
                None
            };
            if json {
                print_json(&json_value!({
                    "deterministic": explanation,
                    "model": model_response
                }))?;
            } else {
                print_explanation(&explanation);
                if let Some(response) = model_response {
                    println!(
                        "\nMODEL — optional explanation ({})\n{}",
                        response.provider, response.text
                    );
                }
            }
        }
        Command::Trace {
            target,
            path,
            depth,
            json,
        } => {
            let trace = zuri_core::trace(&root(path)?, &target, depth)?;
            if json {
                print_json(&trace)?;
            } else {
                for node in trace {
                    println!(
                        "{}{} [{:?}] {}:{}",
                        "  ".repeat(node.depth),
                        node.name,
                        node.resolution,
                        node.file,
                        node.line
                    );
                }
            }
        }
        Command::Review {
            path,
            json,
            severity,
        } => {
            let mut findings = db(&root(path)?)?.findings()?;
            if let Some(severity) = severity {
                let minimum: Severity = severity.into();
                findings.retain(|finding| finding.severity >= minimum);
            }
            if json {
                print_json(&findings)?;
            } else {
                if findings.is_empty() {
                    println!("No current deterministic findings.");
                }
                for finding in findings {
                    println!(
                        "{}  {}  {}:{}\n  {} [{} / {}]\n",
                        finding.id,
                        finding.rule_id,
                        finding.location.file,
                        finding.location.start_line,
                        finding.title,
                        finding.evidence,
                        finding.confidence
                    );
                }
            }
        }
        Command::Why { id, path, json } => {
            let finding = db(&root(path)?)?
                .finding(&id)?
                .context("finding not found")?;
            if json {
                print_json(&finding)?;
            } else {
                println!(
                    "{} — {}\n{}:{}\nSeverity: {}\nEvidence: {} / {}\n\n{}",
                    finding.rule_id,
                    finding.title,
                    finding.location.file,
                    finding.location.start_line,
                    finding.severity,
                    finding.evidence,
                    finding.confidence,
                    finding.description
                );
                if let Some(remediation) = finding.remediation {
                    println!("\nSuggested remediation\n{remediation}");
                }
            }
        }
        Command::Concepts { target, path, json } => {
            let concepts = zuri_core::concepts(&root(path)?, &target)?;
            if json {
                print_json(&concepts)?;
            } else {
                for concept in concepts {
                    println!("- {concept}");
                }
            }
        }
        Command::Search { query, limit, json } => {
            let pack = KnowledgePack::ensure_builtin()?;
            let hits = pack.search(&query, limit)?;
            if json {
                print_json(&hits)?;
            } else {
                for hit in hits {
                    println!(
                        "{} — {}\n  {}\n  source: {} ({})\n",
                        hit.entry.entry_id,
                        hit.entry.title,
                        hit.entry.body,
                        hit.entry.source_url,
                        hit.entry.source_status
                    );
                }
            }
        }
        Command::Learn { topic, json } => {
            let entries = KnowledgePack::ensure_builtin()?.topic(&topic)?;
            if entries.is_empty() {
                anyhow::bail!("no installed Python Core entry for `{topic}`")
            }
            if json {
                print_json(&entries)?;
            } else {
                for entry in entries {
                    println!(
                        "{}\n{}\n\nSource: {}\nStatus: {}\n",
                        entry.title, entry.body, entry.source_url, entry.source_status
                    );
                }
            }
        }
        Command::Quiz {
            target,
            topic,
            path,
            json,
        } => {
            let questions = match (target, topic) {
                (Some(target), None) => zuri_core::quiz(&root(path)?, &target)?,
                (None, Some(topic)) => zuri_core::quiz_topic(&topic)?,
                (Some(_), Some(_)) => {
                    anyhow::bail!("choose a project target or --topic, not both")
                }
                (None, None) => {
                    anyhow::bail!("provide a project target or use `zuri quiz --topic <topic>`")
                }
            };
            if json {
                print_json(&questions)?;
            } else {
                for (index, question) in questions.iter().enumerate() {
                    println!(
                        "Q{}. {}\n    Answer: {}\n    Evidence: {}\n",
                        index + 1,
                        question.question,
                        question.answer,
                        question.evidence
                    );
                }
            }
        }
        Command::VibeCheck { path, json } => {
            let report = zuri_core::vibe_check(&root(path)?)?;
            if json {
                print_json(&report)?;
            } else {
                println!(
                    "Zuri Vibe Check\nReadiness: {}\nErrors: {}  warnings: {}  unresolved calls: {}\nBasis: {}",
                    report.readiness,
                    report.error_findings,
                    report.warning_findings,
                    report.unresolved_calls,
                    report.basis
                );
                if !report.concepts_to_review.is_empty() {
                    println!(
                        "\nConcepts to understand before changing this code\n  {}",
                        report.concepts_to_review.join("\n  ")
                    );
                }
                if !report.priority_findings.is_empty() {
                    println!("\nPriority findings");
                    for finding in &report.priority_findings {
                        println!(
                            "  {} {}:{} — {} [{} / {}]",
                            finding.rule_id,
                            finding.location.file,
                            finding.location.start_line,
                            finding.title,
                            finding.evidence,
                            finding.confidence
                        );
                    }
                }
            }
        }
        Command::Doctor { json } => {
            let report = zuri_core::doctor();
            if json {
                print_json(&report)?;
            } else {
                println!("CodeSage Zuri Doctor");
                for (key, value) in report {
                    println!("{key}: {value}");
                }
            }
        }
        Command::Model { command } => match command {
            ModelCommand::Status => print_json(&ModelConfig::load()?.status())?,
            ModelCommand::Configure {
                endpoint,
                model,
                enabled,
            } => {
                ModelConfig {
                    enabled,
                    endpoint,
                    model,
                }
                .save()?;
                println!(
                    "Saved local model configuration. Deterministic Zuri remains independent."
                );
            }
            ModelCommand::Test => {
                let provider = LocalOpenAiProvider::new(ModelConfig::load()?)?;
                print_json(&provider.health_check())?;
            }
        },
        Command::Tui { path } => tui::run(&root(path)?)?,
    }
    Ok(())
}
