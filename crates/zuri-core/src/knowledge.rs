use crate::{EvidenceKind, Result, ZuriError};
use directories::ProjectDirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const BUILTIN: &str = include_str!("../assets/python_core.json");
const BUILTIN_ID: &str = "python-core";
const BUILTIN_VERSION: &str = "0.1.0";
static PACK_INSTALL_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub entry_id: String,
    pub topic: String,
    pub title: String,
    pub body: String,
    pub keywords: Vec<String>,
    pub source_title: String,
    pub source_url: String,
    pub source_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHit {
    pub entry: KnowledgeEntry,
    pub rank: f64,
    pub evidence: EvidenceKind,
}

pub struct KnowledgePack {
    path: PathBuf,
}

fn fts_query(raw: &str) -> Option<String> {
    let terms: Vec<String> = raw
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

fn pack_is_valid(path: &Path) -> bool {
    let Ok(conn) = Connection::open(path) else {
        return false;
    };
    let tables_ok = ["manifest", "entries", "entries_fts"].iter().all(|table| {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE name=?1 LIMIT 1",
            params![table],
            |_| Ok(()),
        )
        .is_ok()
    });
    if !tables_ok {
        return false;
    }
    let id: rusqlite::Result<String> =
        conn.query_row("SELECT value FROM manifest WHERE key='id'", [], |row| {
            row.get(0)
        });
    let version: rusqlite::Result<String> = conn.query_row(
        "SELECT value FROM manifest WHERE key='version'",
        [],
        |row| row.get(0),
    );
    matches!((id, version), (Ok(id), Ok(version)) if id == BUILTIN_ID && version == BUILTIN_VERSION)
}

fn temp_pack_path(path: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("knowledge.zpk");
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), stamp))
}

impl KnowledgePack {
    pub fn builtin_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("dev", "codesage-zuri", "CodeSage Zuri")
            .ok_or_else(|| ZuriError::Config("cannot determine OS data directory".into()))?;
        let dir = dirs.data_dir().join("packs");
        fs::create_dir_all(&dir)?;
        Ok(dir.join("python-core-0.1.0.zpk"))
    }

    pub fn ensure_builtin() -> Result<Self> {
        let path = Self::builtin_path()?;
        let install_lock = PACK_INSTALL_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = install_lock
            .lock()
            .map_err(|_| ZuriError::Config("knowledge-pack installer lock was poisoned".into()))?;
        if !pack_is_valid(&path) {
            Self::build_from_json_atomic(&path, BUILTIN)?;
        }
        Ok(Self { path })
    }

    pub fn build_from_json(path: &Path, json: &str) -> Result<()> {
        Self::build_from_json_atomic(path, json)
    }

    fn build_from_json_atomic(path: &Path, json: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = temp_pack_path(path);
        if temp.exists() {
            fs::remove_file(&temp)?;
        }
        let build_result = (|| -> Result<()> {
            let mut conn = Connection::open(&temp)?;
            conn.execute_batch(
                "CREATE TABLE manifest(key TEXT PRIMARY KEY,value TEXT NOT NULL);
                 CREATE TABLE entries(entry_id TEXT PRIMARY KEY,topic TEXT NOT NULL,title TEXT NOT NULL,body TEXT NOT NULL,keywords_json TEXT NOT NULL,source_title TEXT NOT NULL,source_url TEXT NOT NULL,source_status TEXT NOT NULL);
                 CREATE VIRTUAL TABLE entries_fts USING fts5(entry_id UNINDEXED,title,topic,body,keywords);",
            )?;
            conn.execute(
                "INSERT INTO manifest(key,value) VALUES('id','python-core'),('version','0.1.0'),('schema_version','1'),('language','python')",
                [],
            )?;
            let entries: Vec<KnowledgeEntry> =
                serde_json::from_str(json).map_err(|e| ZuriError::Config(e.to_string()))?;
            let tx = conn.transaction()?;
            for entry in entries {
                let keywords = entry.keywords.join(" ");
                tx.execute(
                    "INSERT INTO entries VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                    params![
                        entry.entry_id,
                        entry.topic,
                        entry.title,
                        entry.body,
                        serde_json::to_string(&entry.keywords).unwrap(),
                        entry.source_title,
                        entry.source_url,
                        entry.source_status
                    ],
                )?;
                tx.execute(
                    "INSERT INTO entries_fts(entry_id,title,topic,body,keywords) VALUES(?1,?2,?3,?4,?5)",
                    params![entry.entry_id, entry.title, entry.topic, entry.body, keywords],
                )?;
            }
            tx.commit()?;
            drop(conn);
            if !pack_is_valid(&temp) {
                return Err(ZuriError::Config(
                    "generated knowledge pack failed schema validation".into(),
                ));
            }
            if path.exists() {
                fs::remove_file(path)?;
            }
            fs::rename(&temp, path)?;
            Ok(())
        })();
        if build_result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        build_result
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<KnowledgeHit>> {
        let Some(query) = fts_query(query) else {
            return Ok(Vec::new());
        };
        let conn = Connection::open(&self.path)?;
        let mut statement = conn.prepare(
            "SELECT e.entry_id,e.topic,e.title,e.body,e.keywords_json,e.source_title,e.source_url,e.source_status,bm25(entries_fts)
             FROM entries_fts
             JOIN entries e USING(entry_id)
             WHERE entries_fts MATCH ?1
             ORDER BY bm25(entries_fts)
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![query, limit as i64], |row| {
            let keywords: String = row.get(4)?;
            Ok(KnowledgeHit {
                entry: KnowledgeEntry {
                    entry_id: row.get(0)?,
                    topic: row.get(1)?,
                    title: row.get(2)?,
                    body: row.get(3)?,
                    keywords: serde_json::from_str(&keywords).unwrap_or_default(),
                    source_title: row.get(5)?,
                    source_url: row.get(6)?,
                    source_status: row.get(7)?,
                },
                rank: row.get(8)?,
                evidence: EvidenceKind::Documented,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn topic(&self, topic: &str) -> Result<Vec<KnowledgeEntry>> {
        let conn = Connection::open(&self.path)?;
        let mut statement = conn.prepare(
            "SELECT entry_id,topic,title,body,keywords_json,source_title,source_url,source_status
             FROM entries
             WHERE topic=?1 OR entry_id=?1 OR entry_id=?2
             ORDER BY title",
        )?;
        let prefixed = format!("python.{topic}");
        let rows = statement.query_map(params![topic, prefixed], |row| {
            let keywords: String = row.get(4)?;
            Ok(KnowledgeEntry {
                entry_id: row.get(0)?,
                topic: row.get(1)?,
                title: row.get(2)?,
                body: row.get(3)?,
                keywords: serde_json::from_str(&keywords).unwrap_or_default(),
                source_title: row.get(5)?,
                source_url: row.get(6)?,
                source_status: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fts_query_treats_user_punctuation_as_text() {
        assert_eq!(
            fts_query("mutable-defaults").as_deref(),
            Some("\"mutable\" \"defaults\"")
        );
        assert_eq!(
            fts_query("mutable default").as_deref(),
            Some("\"mutable\" \"default\"")
        );
        assert!(fts_query("---").is_none());
    }

    #[test]
    fn invalid_pack_is_detected() {
        let path =
            std::env::temp_dir().join(format!("zuri-invalid-pack-{}.sqlite", std::process::id()));
        let _ = fs::remove_file(&path);
        Connection::open(&path).unwrap();
        assert!(!pack_is_valid(&path));
        let _ = fs::remove_file(path);
    }
}
