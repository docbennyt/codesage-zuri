use crate::{
    parser::{CallResolution, ParsedFile, SourceLocation, Symbol, SymbolKind, SymbolMetrics},
    resolver::{
        resolve_calls as resolve_python_calls, ResolutionCall, ResolutionImport, ResolutionSymbol,
    },
    CallEdgeView, Confidence, DiscoveredFile, EvidenceKind, Result, Severity, ZuriError,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub rule_id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub confidence: Confidence,
    pub evidence: EvidenceKind,
    pub location: SourceLocation,
    pub remediation: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectStats {
    pub files: usize,
    pub symbols: usize,
    pub functions: usize,
    pub classes: usize,
    pub imports: usize,
    pub calls_resolved: usize,
    pub calls_probable: usize,
    pub calls_unresolved: usize,
    pub findings: usize,
}
pub struct ProjectDb {
    conn: Connection,
    path: PathBuf,
    root: PathBuf,
}

const SCHEMA: &str = r#"PRAGMA foreign_keys=ON;
CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY,value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS files(path TEXT PRIMARY KEY,size INTEGER NOT NULL,modified_unix INTEGER NOT NULL,hash TEXT NOT NULL,language TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS symbols(id TEXT PRIMARY KEY,file_path TEXT NOT NULL REFERENCES files(path) ON DELETE CASCADE,kind TEXT NOT NULL,name TEXT NOT NULL,qualified_name TEXT NOT NULL,signature TEXT NOT NULL,parameters_json TEXT NOT NULL,decorators_json TEXT NOT NULL,start_line INTEGER NOT NULL,start_column INTEGER NOT NULL,end_line INTEGER NOT NULL,end_column INTEGER NOT NULL,branches INTEGER NOT NULL,loops INTEGER NOT NULL,returns INTEGER NOT NULL,exception_handlers INTEGER NOT NULL,concepts_json TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);CREATE INDEX IF NOT EXISTS idx_symbols_qualified ON symbols(qualified_name);CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path);
CREATE TABLE IF NOT EXISTS imports(id INTEGER PRIMARY KEY AUTOINCREMENT,file_path TEXT NOT NULL REFERENCES files(path) ON DELETE CASCADE,module TEXT NOT NULL,imported_name TEXT,alias TEXT,line INTEGER NOT NULL);CREATE INDEX IF NOT EXISTS idx_imports_file ON imports(file_path);
CREATE TABLE IF NOT EXISTS calls(id INTEGER PRIMARY KEY AUTOINCREMENT,file_path TEXT NOT NULL REFERENCES files(path) ON DELETE CASCADE,caller_symbol_id TEXT,target_text TEXT NOT NULL,simple_name TEXT,target_symbol_id TEXT,resolution TEXT NOT NULL,line INTEGER NOT NULL);CREATE INDEX IF NOT EXISTS idx_calls_caller ON calls(caller_symbol_id);CREATE INDEX IF NOT EXISTS idx_calls_target ON calls(target_symbol_id);
CREATE TABLE IF NOT EXISTS signals(id INTEGER PRIMARY KEY AUTOINCREMENT,file_path TEXT NOT NULL REFERENCES files(path) ON DELETE CASCADE,rule_id TEXT NOT NULL,title TEXT NOT NULL,description TEXT NOT NULL,severity TEXT NOT NULL,confidence TEXT NOT NULL,evidence TEXT NOT NULL,line INTEGER NOT NULL,remediation TEXT);
CREATE TABLE IF NOT EXISTS findings(id TEXT PRIMARY KEY,file_path TEXT NOT NULL REFERENCES files(path) ON DELETE CASCADE,rule_id TEXT NOT NULL,title TEXT NOT NULL,description TEXT NOT NULL,severity TEXT NOT NULL,confidence TEXT NOT NULL,evidence TEXT NOT NULL,line INTEGER NOT NULL,remediation TEXT);"#;
fn kind_to_str(k: SymbolKind) -> &'static str {
    match k {
        SymbolKind::Function => "function",
        SymbolKind::AsyncFunction => "async_function",
        SymbolKind::Class => "class",
        SymbolKind::Method => "method",
    }
}
fn kind_from_str(s: &str) -> SymbolKind {
    match s {
        "class" => SymbolKind::Class,
        "method" => SymbolKind::Method,
        "async_function" => SymbolKind::AsyncFunction,
        _ => SymbolKind::Function,
    }
}
fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}
fn severity_parse(s: &str) -> Severity {
    match s {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}
fn confidence_str(c: Confidence) -> &'static str {
    match c {
        Confidence::Low => "low",
        Confidence::Medium => "medium",
        Confidence::High => "high",
    }
}
fn confidence_parse(s: &str) -> Confidence {
    match s {
        "high" => Confidence::High,
        "medium" => Confidence::Medium,
        _ => Confidence::Low,
    }
}
fn evidence_str(e: EvidenceKind) -> &'static str {
    match e {
        EvidenceKind::Fact => "fact",
        EvidenceKind::Documented => "documented",
        EvidenceKind::Inference => "inference",
        EvidenceKind::Model => "model",
    }
}
fn evidence_parse(s: &str) -> EvidenceKind {
    match s {
        "documented" => EvidenceKind::Documented,
        "inference" => EvidenceKind::Inference,
        "model" => EvidenceKind::Model,
        _ => EvidenceKind::Fact,
    }
}
fn resolution_parse(s: &str) -> CallResolution {
    match s {
        "resolved" => CallResolution::Resolved,
        "probable" => CallResolution::Probable,
        _ => CallResolution::Unresolved,
    }
}

impl ProjectDb {
    pub fn open(path: &Path, root: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        conn.execute(
            "INSERT OR REPLACE INTO meta(key,value) VALUES('project_root',?1)",
            params![root.to_string_lossy()],
        )?;
        Ok(Self {
            conn,
            path: path.into(),
            root: root.into(),
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn begin(&mut self) -> Result<Transaction<'_>> {
        Ok(self.conn.transaction()?)
    }
    pub fn clear_index(&self) -> Result<()> {
        self.conn.execute_batch("DELETE FROM findings;DELETE FROM signals;DELETE FROM calls;DELETE FROM imports;DELETE FROM symbols;DELETE FROM files;")?;
        Ok(())
    }
    pub fn file_hashes(&self) -> Result<HashMap<String, String>> {
        let mut st = self.conn.prepare("SELECT path,hash FROM files")?;
        let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<HashMap<_, _>>>()?)
    }
    pub fn resolve_calls(&self) -> Result<()> {
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
    pub fn rebuild_findings(&self) -> Result<()> {
        self.conn.execute("DELETE FROM findings", [])?;
        let mut st=self.conn.prepare("SELECT file_path,rule_id,title,description,severity,confidence,evidence,line,remediation FROM signals")?;
        let mut rows = st.query([])?;
        while let Some(r) = rows.next()? {
            let file: String = r.get(0)?;
            let rule: String = r.get(1)?;
            let line: i64 = r.get(7)?;
            let id = blake3::hash(format!("{rule}\0{file}\0{line}").as_bytes()).to_hex()[..16]
                .to_string();
            self.conn.execute("INSERT OR REPLACE INTO findings(id,file_path,rule_id,title,description,severity,confidence,evidence,line,remediation) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![id,file,rule,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?,line,r.get::<_,Option<String>>(8)?])?;
        }
        Ok(())
    }
    pub fn stats(&self) -> Result<ProjectStats> {
        let count = |sql: &str| -> Result<usize> {
            Ok(self.conn.query_row(sql, [], |r| r.get::<_, i64>(0))? as usize)
        };
        Ok(ProjectStats {
            files: count("SELECT COUNT(*) FROM files")?,
            symbols: count("SELECT COUNT(*) FROM symbols")?,
            functions: count(
                "SELECT COUNT(*) FROM symbols WHERE kind IN ('function','async_function','method')",
            )?,
            classes: count("SELECT COUNT(*) FROM symbols WHERE kind='class'")?,
            imports: count("SELECT COUNT(*) FROM imports")?,
            calls_resolved: count("SELECT COUNT(*) FROM calls WHERE resolution='resolved'")?,
            calls_probable: count("SELECT COUNT(*) FROM calls WHERE resolution='probable'")?,
            calls_unresolved: count("SELECT COUNT(*) FROM calls WHERE resolution='unresolved'")?,
            findings: count("SELECT COUNT(*) FROM findings")?,
        })
    }
    pub fn symbols(&self, query: Option<&str>) -> Result<Vec<Symbol>> {
        let base="SELECT id,kind,name,qualified_name,signature,parameters_json,decorators_json,file_path,start_line,start_column,end_line,end_column,branches,loops,returns,exception_handlers,concepts_json FROM symbols";
        let mut out = Vec::new();
        if let Some(q) = query {
            let mut st = self.conn.prepare(&format!(
                "{base} WHERE name LIKE ?1 OR qualified_name LIKE ?1 ORDER BY file_path,start_line"
            ))?;
            let pattern = format!("%{q}%");
            let rows = st.query_map(params![pattern], symbol_from_row)?;
            for r in rows {
                out.push(r?);
            }
        } else {
            let mut st = self
                .conn
                .prepare(&format!("{base} ORDER BY file_path,start_line"))?;
            let rows = st.query_map([], symbol_from_row)?;
            for r in rows {
                out.push(r?);
            }
        }
        Ok(out)
    }
    pub fn find_symbol(&self, target: &str) -> Result<Option<Symbol>> {
        let all = self.symbols(None)?;
        if let Some((file, name)) = target.split_once("::") {
            return Ok(all.into_iter().find(|s| {
                s.location.file == file && (s.qualified_name == name || s.name == name)
            }));
        }
        let exact: Vec<Symbol> = all
            .into_iter()
            .filter(|s| s.qualified_name == target || s.name == target)
            .collect();
        if exact.len() == 1 {
            Ok(exact.into_iter().next())
        } else if exact.is_empty() {
            Ok(None)
        } else {
            Err(ZuriError::Config(format!(
                "symbol `{target}` is ambiguous; use file.py::qualified.name"
            )))
        }
    }
    pub fn callers(&self, id: &str) -> Result<Vec<CallEdgeView>> {
        let mut st = self.conn.prepare(
            "SELECT COALESCE(s.qualified_name, '<module>'), c.file_path, c.line, c.resolution, c.caller_symbol_id \
             FROM calls c LEFT JOIN symbols s ON s.id = c.caller_symbol_id \
             WHERE c.target_symbol_id=?1 ORDER BY c.file_path,c.line",
        )?;
        let rows = st.query_map(params![id], |r| {
            Ok(CallEdgeView {
                name: r.get(0)?,
                file: r.get(1)?,
                line: r.get::<_, i64>(2)? as usize,
                resolution: resolution_parse(&r.get::<_, String>(3)?),
                symbol_id: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
    pub fn callees(&self, id: &str) -> Result<Vec<CallEdgeView>> {
        let mut st=self.conn.prepare("SELECT target_text,file_path,line,resolution,target_symbol_id FROM calls WHERE caller_symbol_id=?1 ORDER BY line")?;
        let rows = st.query_map(params![id], |r| {
            Ok(CallEdgeView {
                name: r.get(0)?,
                file: r.get(1)?,
                line: r.get::<_, i64>(2)? as usize,
                resolution: resolution_parse(&r.get::<_, String>(3)?),
                symbol_id: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
    pub fn findings(&self) -> Result<Vec<Finding>> {
        let mut st=self.conn.prepare("SELECT id,file_path,rule_id,title,description,severity,confidence,evidence,line,remediation FROM findings ORDER BY file_path,line")?;
        let rows = st.query_map([], finding_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
    pub fn finding(&self, id: &str) -> Result<Option<Finding>> {
        Ok(self.conn.query_row("SELECT id,file_path,rule_id,title,description,severity,confidence,evidence,line,remediation FROM findings WHERE id=?1",params![id],finding_from_row).optional()?)
    }
    pub fn findings_for_file(&self, file: &str) -> Result<Vec<Finding>> {
        let mut st=self.conn.prepare("SELECT id,file_path,rule_id,title,description,severity,confidence,evidence,line,remediation FROM findings WHERE file_path=?1 ORDER BY line")?;
        let rows = st.query_map(params![file], finding_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
fn symbol_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Symbol> {
    let file: String = r.get(7)?;
    Ok(Symbol {
        id: r.get(0)?,
        kind: kind_from_str(&r.get::<_, String>(1)?),
        name: r.get(2)?,
        qualified_name: r.get(3)?,
        signature: r.get(4)?,
        parameters: serde_json::from_str(&r.get::<_, String>(5)?).unwrap_or_default(),
        decorators: serde_json::from_str(&r.get::<_, String>(6)?).unwrap_or_default(),
        location: SourceLocation {
            file,
            start_line: r.get::<_, i64>(8)? as usize,
            start_column: r.get::<_, i64>(9)? as usize,
            end_line: r.get::<_, i64>(10)? as usize,
            end_column: r.get::<_, i64>(11)? as usize,
        },
        metrics: SymbolMetrics {
            branches: r.get::<_, i64>(12)? as usize,
            loops: r.get::<_, i64>(13)? as usize,
            returns: r.get::<_, i64>(14)? as usize,
            exception_handlers: r.get::<_, i64>(15)? as usize,
        },
        concepts: serde_json::from_str(&r.get::<_, String>(16)?).unwrap_or_default(),
    })
}
fn finding_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Finding> {
    let file: String = r.get(1)?;
    let line = r.get::<_, i64>(8)? as usize;
    Ok(Finding {
        id: r.get(0)?,
        rule_id: r.get(2)?,
        title: r.get(3)?,
        description: r.get(4)?,
        severity: severity_parse(&r.get::<_, String>(5)?),
        confidence: confidence_parse(&r.get::<_, String>(6)?),
        evidence: evidence_parse(&r.get::<_, String>(7)?),
        location: SourceLocation {
            file,
            start_line: line,
            start_column: 1,
            end_line: line,
            end_column: 1,
        },
        remediation: r.get(9)?,
    })
}

pub(crate) fn replace_file_in_tx(
    tx: &Transaction<'_>,
    file: &DiscoveredFile,
    p: &ParsedFile,
) -> Result<()> {
    let rel = file.relative_path.to_string_lossy().replace('\\', "/");
    tx.execute("DELETE FROM files WHERE path=?1", params![rel])?;
    tx.execute(
        "INSERT INTO files(path,size,modified_unix,hash,language) VALUES(?1,?2,?3,?4,'python')",
        params![rel, file.size as i64, file.modified_unix as i64, file.hash],
    )?;
    for s in &p.symbols {
        tx.execute("INSERT INTO symbols VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",params![s.id,rel,kind_to_str(s.kind),s.name,s.qualified_name,s.signature,serde_json::to_string(&s.parameters).unwrap(),serde_json::to_string(&s.decorators).unwrap(),s.location.start_line as i64,s.location.start_column as i64,s.location.end_line as i64,s.location.end_column as i64,s.metrics.branches as i64,s.metrics.loops as i64,s.metrics.returns as i64,s.metrics.exception_handlers as i64,serde_json::to_string(&s.concepts).unwrap()])?;
    }
    for i in &p.imports {
        tx.execute(
            "INSERT INTO imports(file_path,module,imported_name,alias,line) VALUES(?1,?2,?3,?4,?5)",
            params![rel, i.module, i.imported_name, i.alias, i.line as i64],
        )?;
    }
    for c in &p.calls {
        tx.execute("INSERT INTO calls(file_path,caller_symbol_id,target_text,simple_name,target_symbol_id,resolution,line) VALUES(?1,?2,?3,?4,NULL,'unresolved',?5)",params![rel,c.caller_symbol_id,c.target_text,c.simple_name,c.line as i64])?;
    }
    for s in &p.signals {
        tx.execute("INSERT INTO signals(file_path,rule_id,title,description,severity,confidence,evidence,line,remediation) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![rel,s.rule_id,s.title,s.description,severity_str(s.severity),confidence_str(s.confidence),evidence_str(s.evidence),s.line as i64,s.remediation])?;
    }
    Ok(())
}
pub(crate) fn remove_missing_in_tx(tx: &Transaction<'_>, seen: &HashSet<String>) -> Result<usize> {
    let mut st = tx.prepare("SELECT path FROM files")?;
    let existing: Vec<String> = st
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    drop(st);
    let mut removed = 0;
    for p in existing {
        if !seen.contains(&p) {
            tx.execute("DELETE FROM files WHERE path=?1", params![p])?;
            removed += 1;
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evidence_names() {
        assert_eq!(evidence_str(EvidenceKind::Fact), "fact");
        assert_eq!(evidence_parse("inference"), EvidenceKind::Inference);
    }
}
