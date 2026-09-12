use crate::{index_project, open_project, KnowledgePack, ProjectStats, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub root: PathBuf,
    pub rounds: usize,
    pub platform: String,
    pub architecture: String,
    pub cpu_threads: usize,
    pub cold_index_us: u128,
    pub incremental_index_us: Vec<u128>,
    pub incremental_median_us: u128,
    pub symbol_lookup_us: Vec<u128>,
    pub symbol_lookup_median_us: Option<u128>,
    pub knowledge_search_us: Vec<u128>,
    pub knowledge_search_median_us: u128,
    pub index_bytes: u64,
    pub peak_rss_kib: Option<u64>,
    pub stats: ProjectStats,
    pub network_used: bool,
    pub model_used: bool,
    pub notes: Vec<String>,
}

fn median(values: &[u128]) -> Option<u128> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        Some((sorted[middle - 1] + sorted[middle]) / 2)
    } else {
        Some(sorted[middle])
    }
}

#[cfg(target_os = "linux")]
fn peak_rss_kib() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let value = line.strip_prefix("VmHWM:")?.trim();
        value
            .split_whitespace()
            .next()
            .and_then(|number| number.parse().ok())
    })
}

#[cfg(not(target_os = "linux"))]
fn peak_rss_kib() -> Option<u64> {
    None
}

pub fn benchmark_project(root: &Path, rounds: usize) -> Result<BenchmarkReport> {
    let root = fs::canonicalize(root)?;
    let rounds = rounds.clamp(1, 50);

    let started = Instant::now();
    let cold = index_project(&root, true)?;
    let cold_index_us = started.elapsed().as_micros();

    let mut incremental_index_us = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let started = Instant::now();
        let _ = index_project(&root, false)?;
        incremental_index_us.push(started.elapsed().as_micros());
    }

    let db = open_project(&root)?;
    let symbols = db.symbols(None)?;
    let first_symbol = symbols.first().map(|symbol| symbol.qualified_name.clone());
    let mut symbol_lookup_us = Vec::new();
    if let Some(target) = first_symbol {
        for _ in 0..rounds {
            let started = Instant::now();
            let _ = db.find_symbol(&target)?;
            symbol_lookup_us.push(started.elapsed().as_micros());
        }
    }

    let pack = KnowledgePack::ensure_builtin()?;
    let mut knowledge_search_us = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let started = Instant::now();
        let _ = pack.search("mutable default", 5)?;
        knowledge_search_us.push(started.elapsed().as_micros());
    }

    let index_path = db.path().to_path_buf();
    let index_bytes = fs::metadata(index_path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let stats = db.stats()?;
    let rss = peak_rss_kib();
    let mut notes = vec![
        "Cold indexing deliberately rebuilds Zuri's local cache; project source files are never executed or modified.".into(),
        "GitHub-hosted CI is a portability check, not evidence that 4 GB or 8 GB hardware targets are met.".into(),
    ];
    if rss.is_none() {
        notes.push(
            "Peak RSS is currently reported only on Linux via /proc/self/status VmHWM; use an OS-native profiler for Windows/macOS memory measurements."
                .into(),
        );
    }

    Ok(BenchmarkReport {
        root,
        rounds,
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        cpu_threads: std::thread::available_parallelism()
            .map(|threads| threads.get())
            .unwrap_or(1),
        cold_index_us,
        incremental_median_us: median(&incremental_index_us).unwrap_or(0),
        incremental_index_us,
        symbol_lookup_median_us: median(&symbol_lookup_us),
        symbol_lookup_us,
        knowledge_search_median_us: median(&knowledge_search_us).unwrap_or(0),
        knowledge_search_us,
        index_bytes,
        peak_rss_kib: rss,
        stats: if cold.stats.files == 0 {
            cold.stats
        } else {
            stats
        },
        network_used: false,
        model_used: false,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_handles_even_and_odd_samples() {
        assert_eq!(median(&[3, 1, 2]), Some(2));
        assert_eq!(median(&[1, 4, 2, 3]), Some(2));
        assert_eq!(median(&[]), None);
    }
}
