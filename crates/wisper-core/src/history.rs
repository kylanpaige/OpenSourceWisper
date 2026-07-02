//! Local dictation history, stored as JSON Lines in the app data dir.
//!
//! History never leaves the machine. Each entry keeps both the raw ASR output
//! and the final injected text so users can audit what the cleanup stage did.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    /// Raw transcript straight from the ASR model.
    pub raw: String,
    /// Final text after dictionary + formatting + LLM cleanup.
    pub final_text: String,
    /// App the text was injected into, if known.
    pub app: String,
    pub duration_ms: u64,
    /// Whether the LLM cleanup stage ran (vs deterministic formatting only).
    pub cleaned: bool,
}

pub fn append(path: &Path, entry: &HistoryEntry) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(entry).map_err(std::io::Error::other)?;
    writeln!(f, "{line}")
}

/// Load up to `limit` most-recent entries (newest first). Corrupt lines are skipped.
pub fn load_recent(path: &Path, limit: usize) -> Vec<HistoryEntry> {
    let Ok(f) = File::open(path) else {
        return Vec::new();
    };
    let mut entries: Vec<HistoryEntry> = BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter_map(|l| serde_json::from_str(&l).ok())
        .collect();
    entries.reverse();
    entries.truncate(limit);
    entries
}

/// Rewrite the file keeping only the newest `keep` entries. Returns entries kept.
pub fn compact(path: &Path, keep: usize) -> std::io::Result<usize> {
    let recent = load_recent(path, keep);
    let mut f = File::create(path)?;
    for e in recent.iter().rev() {
        let line = serde_json::to_string(e).map_err(std::io::Error::other)?;
        writeln!(f, "{line}")?;
    }
    Ok(recent.len())
}

pub fn clear(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(text: &str) -> HistoryEntry {
        HistoryEntry {
            timestamp: Utc::now(),
            raw: format!("um {text}"),
            final_text: text.into(),
            app: "Test".into(),
            duration_ms: 1200,
            cleaned: true,
        }
    }

    #[test]
    fn append_load_compact_clear() {
        let dir = std::env::temp_dir().join(format!("wisper-hist-{}", std::process::id()));
        let path = dir.join("history.jsonl");
        let _ = std::fs::remove_dir_all(&dir);

        for i in 0..5 {
            append(&path, &entry(&format!("entry {i}"))).unwrap();
        }
        let recent = load_recent(&path, 3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].final_text, "entry 4"); // newest first

        assert_eq!(compact(&path, 2).unwrap(), 2);
        assert_eq!(load_recent(&path, 10).len(), 2);

        clear(&path).unwrap();
        assert!(load_recent(&path, 10).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_lines_are_skipped() {
        let dir = std::env::temp_dir().join(format!("wisper-hist-c-{}", std::process::id()));
        let path = dir.join("history.jsonl");
        let _ = std::fs::remove_dir_all(&dir);
        append(&path, &entry("good")).unwrap();
        {
            let mut f = OpenOptions::new().append(true).open(&path).unwrap();
            writeln!(f, "{{not json").unwrap();
        }
        append(&path, &entry("also good")).unwrap();
        assert_eq!(load_recent(&path, 10).len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
