use crate::{Result, ZuriError};
use blake3::hash;
use ignore::{DirEntry, WalkBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredFile {
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub size: u64,
    pub modified_unix: u64,
    pub hash: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectScanner { pub max_file_size: u64 }
impl Default for ProjectScanner { fn default() -> Self { Self { max_file_size: 2 * 1024 * 1024 } } }

fn excluded(entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|t| t.is_dir()) && matches!(entry.file_name().to_string_lossy().as_ref(), ".git"|"node_modules"|"target"|"dist"|"build"|"coverage"|".venv"|"venv"|"env"|"__pycache__"|"vendor")
}
fn language(path: &Path) -> Option<String> { match path.extension().and_then(|s|s.to_str()).map(|s|s.to_ascii_lowercase()) { Some(ext) if ext=="py"||ext=="pyw"=>Some("python".into()), _=>None } }
fn looks_binary(bytes:&[u8])->bool { bytes.iter().take(8192).any(|b|*b==0) }

impl ProjectScanner {
    pub fn scan(&self, root:&Path)->Result<Vec<DiscoveredFile>> {
        let root=fs::canonicalize(root)?; let mut builder=WalkBuilder::new(&root); builder.standard_filters(true).hidden(false); let mut out=Vec::new();
        for item in builder.build().filter_entry(|e|!excluded(e)) {
            let entry=item.map_err(|e|ZuriError::Config(e.to_string()))?;
            if !entry.file_type().is_some_and(|t|t.is_file()){continue}
            let meta=entry.metadata().map_err(|e|ZuriError::Config(e.to_string()))?; if meta.len()>self.max_file_size{continue}
            let bytes=fs::read(entry.path())?; if looks_binary(&bytes){continue}
            let rel=entry.path().strip_prefix(&root).map_err(|e|ZuriError::Config(e.to_string()))?.to_path_buf();
            let modified=meta.modified().ok().and_then(|m|m.duration_since(UNIX_EPOCH).ok()).map(|d|d.as_secs()).unwrap_or(0);
            out.push(DiscoveredFile{absolute_path:entry.path().to_path_buf(),relative_path:rel,size:meta.len(),modified_unix:modified,hash:hash(&bytes).to_hex().to_string(),language:language(entry.path())});
        }
        out.sort_by(|a,b|a.relative_path.cmp(&b.relative_path)); Ok(out)
    }
}

#[cfg(test)] mod tests { use super::*; #[test] fn deliberately_narrow(){assert_eq!(language(Path::new("a.py")).as_deref(),Some("python"));assert_eq!(language(Path::new("a.js")),None);} }
