//! Personal dictionary: custom words, proper nouns, and text replacements.
//!
//! Two jobs, mirroring Wispr Flow's dictionary:
//! 1. Corrections — "jira" -> "Jira", "open source wisper" -> "OpenSourceWisper".
//! 2. Snippets — a spoken trigger expands to longer text ("my address" -> full address).

use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DictionaryEntry {
    /// What the ASR tends to produce (matched case-insensitively on word boundaries).
    pub spoken: String,
    /// What should be written instead.
    pub written: String,
    pub enabled: bool,
}

impl DictionaryEntry {
    pub fn new(spoken: impl Into<String>, written: impl Into<String>) -> Self {
        Self {
            spoken: spoken.into(),
            written: written.into(),
            enabled: true,
        }
    }
}

/// Apply all enabled entries to `text`. Longer spoken forms are applied first so
/// "open source wisper" wins over a shorter "wisper" entry covering the same span.
pub fn apply(entries: &[DictionaryEntry], text: &str) -> String {
    let mut ordered: Vec<&DictionaryEntry> = entries
        .iter()
        .filter(|e| e.enabled && !e.spoken.trim().is_empty())
        .collect();
    ordered.sort_by_key(|e| std::cmp::Reverse(e.spoken.len()));

    let mut out = text.to_string();
    for entry in ordered {
        let pattern = format!(r"\b{}\b", regex::escape(entry.spoken.trim()));
        let Ok(re) = RegexBuilder::new(&pattern).case_insensitive(true).build() else {
            continue;
        };
        out = re.replace_all(&out, entry.written.as_str()).into_owned();
    }
    out
}

/// The dictionary terms worth hinting to the cleanup LLM (so it also respects them).
pub fn terms_for_prompt(entries: &[DictionaryEntry], max: usize) -> Vec<String> {
    entries
        .iter()
        .filter(|e| e.enabled)
        .take(max)
        .map(|e| e.written.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_case_insensitively_on_word_boundaries() {
        let entries = vec![DictionaryEntry::new("jira", "Jira")];
        assert_eq!(apply(&entries, "check jira and JIRA now"), "check Jira and Jira now");
        // No mid-word replacement.
        assert_eq!(apply(&entries, "jirafication"), "jirafication");
    }

    #[test]
    fn longer_entries_win() {
        let entries = vec![
            DictionaryEntry::new("wisper", "Wisper"),
            DictionaryEntry::new("open source wisper", "OpenSourceWisper"),
        ];
        assert_eq!(
            apply(&entries, "I love open source wisper"),
            "I love OpenSourceWisper"
        );
    }

    #[test]
    fn disabled_entries_are_skipped() {
        let mut e = DictionaryEntry::new("acme", "ACME Corp");
        e.enabled = false;
        assert_eq!(apply(&[e], "call acme"), "call acme");
    }

    #[test]
    fn snippet_expansion() {
        let entries = vec![DictionaryEntry::new(
            "my signature",
            "Best regards,\nKylan",
        )];
        assert_eq!(apply(&entries, "my signature"), "Best regards,\nKylan");
    }
}
