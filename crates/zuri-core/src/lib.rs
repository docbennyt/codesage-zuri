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
pub use model::{ModelConfig, ModelStatus};
pub use parser::{CallResolution, ParsedFile, ReviewSignal, SourceLocation, Symbol, SymbolKind};
pub use scanner::{DiscoveredFile, ProjectScanner};
pub use store::{Finding, ProjectDb, ProjectStats};

#[derive(Debug, thiserror::Error)]
pub enum ZuriError {
    #[error("I/O error: {0}")] Io(#[from] std::io::Error),
    #[error("database error: {0}")] Database(#[from] rusqlite::Error),
    #[error("parse error: {0}")] Parse(String),
    #[error("configuration error: {0}")] Config(String),
    #[error("not found: {0}")] NotFound(String),
}
pub type Result<T> = std::result::Result<T, ZuriError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)] pub enum EvidenceKind { Fact, Documented, Inference, Model }
impl std::fmt::Display for EvidenceKind { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"{}",match self{Self::Fact=>"FACT",Self::Documented=>"DOCUMENTED",Self::Inference=>"INFERENCE",Self::Model=>"MODEL"})}}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)] pub enum Confidence{Low,Medium,High}
impl std::fmt::Display for Confidence{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"{}",match self{Self::Low=>"low",Self::Medium=>"medium",Self::High=>"high"})}}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)] pub enum Severity{Info,Warning,Error}
impl std::fmt::Display for Severity{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{write!(f,"{}",match self{Self::Info=>"info",Self::Warning=>"warning",Self::Error=>"error"})}}

#[derive(Debug, Clone, Serialize, Deserialize)] pub struct IndexReport{pub root:PathBuf,pub discovered_files:usize,pub python_files:usize,pub parsed_files:usize,pub unchanged_files:usize,pub removed_files:usize,pub stats:ProjectStats}
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct CallEdgeView{pub name:String,pub file:String,pub line:usize,pub resolution:CallResolution,pub symbol_id:Option<String>}
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct Explanation{pub symbol:Symbol,pub callers:Vec<CallEdgeView>,pub callees:Vec<CallEdgeView>,pub findings:Vec<Finding>,pub evidence_summary:BTreeMap<String,usize>}
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct TraceNode{pub name:String,pub file:String,pub line:usize,pub resolution:CallResolution,pub depth:usize}
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct QuizQuestion{pub question:String,pub answer:String,pub evidence:EvidenceKind}

fn project_key(root:&Path)->String{let mut h=Hasher::new();h.update(root.to_string_lossy().as_bytes());h.finalize().to_hex()[..16].to_string()}
pub fn project_db_path(root:&Path)->Result<PathBuf>{let root=fs::canonicalize(root)?;let dirs=ProjectDirs::from("dev","codesage-zuri","CodeSage Zuri").ok_or_else(||ZuriError::Config("cannot determine OS data directory".into()))?;let dir=dirs.cache_dir().join("projects").join(project_key(&root));fs::create_dir_all(&dir)?;Ok(dir.join("index.sqlite"))}
pub fn open_project(root:&Path)->Result<ProjectDb>{let root=fs::canonicalize(root)?;ProjectDb::open(&project_db_path(&root)?,&root)}

pub fn index_project(root:&Path,rebuild:bool)->Result<IndexReport>{
 let root=fs::canonicalize(root)?;let discovered=ProjectScanner::default().scan(&root)?;let mut db=ProjectDb::open(&project_db_path(&root)?,&root)?;if rebuild{db.clear_index()?;}let previous=db.file_hashes()?;let mut seen=HashSet::new();let mut parsed=0;let mut unchanged=0;let python_files=discovered.iter().filter(|f|f.language.as_deref()==Some("python")).count();let tx=db.begin()?;
 for file in &discovered{if file.language.as_deref()!=Some("python"){continue}let rel=file.relative_path.to_string_lossy().replace('\\',"/");seen.insert(rel.clone());if previous.get(&rel).is_some_and(|h|h==&file.hash){unchanged+=1;continue}let source=fs::read_to_string(&file.absolute_path).map_err(|e|ZuriError::Parse(format!("{rel}: {e}")))?;let parsed_file=parser::parse_python(&rel,&source)?;store::replace_file_in_tx(&tx,file,&parsed_file)?;parsed+=1;}
 let removed=store::remove_missing_in_tx(&tx,&seen)?;tx.commit()?;db.resolve_calls()?;db.rebuild_findings()?;let stats=db.stats()?;Ok(IndexReport{root,discovered_files:discovered.len(),python_files,parsed_files:parsed,unchanged_files:unchanged,removed_files:removed,stats})
}

pub fn explain(root:&Path,target:&str)->Result<Explanation>{let db=open_project(root)?;let symbol=db.find_symbol(target)?.ok_or_else(||ZuriError::NotFound(target.into()))?;let callers=db.callers(&symbol.id)?;let callees=db.callees(&symbol.id)?;let findings=db.findings_for_file(&symbol.location.file)?;let mut evidence=BTreeMap::new();*evidence.entry("FACT".into()).or_insert(0)+=1+callers.len()+callees.len();for f in &findings{*evidence.entry(f.evidence.to_string()).or_insert(0)+=1;}Ok(Explanation{symbol,callers,callees,findings,evidence_summary:evidence})}
pub fn trace(root:&Path,target:&str,depth:usize)->Result<Vec<TraceNode>>{let db=open_project(root)?;let start=db.find_symbol(target)?.ok_or_else(||ZuriError::NotFound(target.into()))?;let mut result=Vec::new();let mut frontier=vec![(start.id,0usize)];let mut visited=HashSet::new();while let Some((id,d))=frontier.pop(){if d>=depth||!visited.insert(id.clone()){continue}for edge in db.callees(&id)?{result.push(TraceNode{name:edge.name.clone(),file:edge.file.clone(),line:edge.line,resolution:edge.resolution,depth:d+1});if let Some(next)=edge.symbol_id{frontier.push((next,d+1));}}}Ok(result)}
pub fn concepts(root:&Path,target:&str)->Result<Vec<String>>{let db=open_project(root)?;Ok(db.find_symbol(target)?.ok_or_else(||ZuriError::NotFound(target.into()))?.concepts)}
pub fn quiz(root:&Path,target:&str)->Result<Vec<QuizQuestion>>{let db=open_project(root)?;let s=db.find_symbol(target)?.ok_or_else(||ZuriError::NotFound(target.into()))?;let mut q=vec![QuizQuestion{question:format!("Where is `{}` defined?",s.qualified_name),answer:format!("{}:{}",s.location.file,s.location.start_line),evidence:EvidenceKind::Fact},QuizQuestion{question:format!("How many explicit return statements are inside `{}`?",s.qualified_name),answer:s.metrics.returns.to_string(),evidence:EvidenceKind::Fact}];if let Some(call)=db.callees(&s.id)?.first(){q.push(QuizQuestion{question:format!("Name one callable used by `{}`.",s.qualified_name),answer:call.name.clone(),evidence:EvidenceKind::Fact});}Ok(q)}
pub fn doctor()->BTreeMap<String,String>{let mut out=BTreeMap::new();out.insert("platform".into(),std::env::consts::OS.into());out.insert("architecture".into(),std::env::consts::ARCH.into());out.insert("cpu_threads".into(),std::thread::available_parallelism().map(|n|n.get()).unwrap_or(1).to_string());out.insert("network_required".into(),"no".into());out.insert("model_required".into(),"no".into());out.insert("profile".into(),"ECO-compatible deterministic core".into());out}
