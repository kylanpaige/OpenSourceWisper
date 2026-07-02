//! Deterministic (non-LLM) transcript post-processing.
//!
//! This is the always-available fallback that makes raw Whisper output feel
//! polished even when the cleanup LLM is disabled or unreachable: filler
//! removal, optional spoken punctuation commands, whitespace/capitalization
//! tidying.

use crate::settings::FormatSettings;
use regex::Regex;
use std::sync::OnceLock;

static FILLER_RE: OnceLock<Regex> = OnceLock::new();

fn filler_re() -> &'static Regex {
    FILLER_RE.get_or_init(|| {
        // Whisper renders fillers as um/uh/uhm/erm/er/ah/hmm, often followed by a comma.
        Regex::new(r"(?i)\b(um+|uh+|uhm|erm|hmm+)\b[,.]?\s*").unwrap()
    })
}

/// Spoken punctuation commands, longest-first so "question mark" beats "question".
const SPOKEN_PUNCTUATION: &[(&str, &str)] = &[
    ("exclamation mark", "!"),
    ("exclamation point", "!"),
    ("question mark", "?"),
    ("new paragraph", "\n\n"),
    ("new line", "\n"),
    ("open quote", "\u{201C}"),
    ("close quote", "\u{201D}"),
    ("semicolon", ";"),
    ("colon", ":"),
    ("comma", ","),
    ("period", "."),
    ("full stop", "."),
    ("dash", " \u{2014} "),
    ("hyphen", "-"),
    ("open paren", "("),
    ("close paren", ")"),
];

pub fn post_process(text: &str, cfg: &FormatSettings) -> String {
    let mut out = text.trim().to_string();

    if cfg.remove_fillers {
        out = remove_fillers(&out);
    }
    if cfg.spoken_punctuation {
        out = apply_spoken_punctuation(&out);
    }
    out = tidy_whitespace(&out);
    if cfg.smart_capitalization {
        out = capitalize_sentences(&out);
    }
    out
}

pub fn remove_fillers(text: &str) -> String {
    filler_re().replace_all(text, "").into_owned()
}

fn apply_spoken_punctuation(text: &str) -> String {
    let mut out = text.to_string();
    for (spoken, symbol) in SPOKEN_PUNCTUATION {
        let pattern = format!(r"(?i)[,.]?\s*\b{}\b[,.]?", regex::escape(spoken));
        let re = Regex::new(&pattern).unwrap();
        // Attach punctuation to the preceding word; newlines stand alone.
        let replacement = if symbol.starts_with('\n') || symbol.starts_with(" \u{2014}") {
            symbol.to_string()
        } else if matches!(*symbol, "(" | "\u{201C}") {
            format!(" {symbol}")
        } else {
            symbol.to_string()
        };
        out = re.replace_all(&out, replacement.as_str()).into_owned();
    }
    out
}

fn tidy_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // Collapse runs of spaces/tabs but keep intentional newlines.
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let mut last_space = true; // trims leading spaces
        for ch in line.chars() {
            if ch == ' ' || ch == '\t' {
                if !last_space {
                    out.push(' ');
                    last_space = true;
                }
            } else {
                out.push(ch);
                last_space = false;
            }
        }
        while out.ends_with(' ') {
            out.pop();
        }
    }
    // No space before closing punctuation.
    let re = Regex::new(r"\s+([,.!?;:])").unwrap();
    re.replace_all(&out, "$1").into_owned()
}

fn capitalize_sentences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut capitalize_next = true;
    for ch in text.chars() {
        if capitalize_next && ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            out.push(ch);
            if matches!(ch, '.' | '!' | '?' | '\n') {
                capitalize_next = true;
            } else if !ch.is_whitespace() && ch != '"' && ch != '\u{201C}' && ch != '\u{201D}' {
                capitalize_next = false;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(fillers: bool, spoken: bool, caps: bool) -> FormatSettings {
        FormatSettings {
            remove_fillers: fillers,
            spoken_punctuation: spoken,
            smart_capitalization: caps,
        }
    }

    #[test]
    fn strips_fillers() {
        let got = post_process(
            "Um, I think, uh, we should ship it.",
            &cfg(true, false, true),
        );
        assert_eq!(got, "I think, we should ship it.");
    }

    #[test]
    fn keeps_fillers_when_disabled() {
        let got = post_process("Um, hello", &cfg(false, false, false));
        assert_eq!(got, "Um, hello");
    }

    #[test]
    fn spoken_punctuation_commands() {
        let got = post_process(
            "send the report period did you get it question mark",
            &cfg(false, true, true),
        );
        assert_eq!(got, "Send the report. Did you get it?");
    }

    #[test]
    fn new_line_command() {
        let got = post_process("first item new line second item", &cfg(false, true, false));
        assert_eq!(got, "first item\nsecond item");
    }

    #[test]
    fn tidies_whitespace_and_capitalizes() {
        let got = post_process("hello   world .  next sentence", &cfg(false, false, true));
        assert_eq!(got, "Hello world. Next sentence");
    }

    #[test]
    fn does_not_eat_the_word_periodic() {
        let got = post_process("the periodic table", &cfg(false, true, false));
        assert_eq!(got, "the periodic table");
    }
}
