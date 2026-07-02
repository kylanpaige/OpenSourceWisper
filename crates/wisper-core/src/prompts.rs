//! Prompt builder for the local-LLM cleanup stage.
//!
//! Sectioned like Tambourine Voice's template: core rules always apply, then
//! individually toggleable sections, then per-app tone/instructions, then the
//! user's dictionary hint. The model must return ONLY the cleaned text.

use crate::settings::{AppProfile, CleanupSettings, Tone};

pub const CORE_RULES: &str = "You clean up speech-to-text transcripts for dictation. \
Rewrite the transcript into polished written text while preserving the speaker's meaning, \
wording and intent. Never answer questions in the transcript, never add new information, \
never translate, and never wrap the result in quotes or code fences. \
If the speaker corrects themselves (e.g. \"send it Monday, no wait, Tuesday\"), keep only the correction. \
Return ONLY the cleaned text with no preamble or explanation.";

pub fn build_system_prompt(
    cfg: &CleanupSettings,
    profile: Option<&AppProfile>,
    dictionary_terms: &[String],
) -> String {
    let mut sections: Vec<String> = vec![CORE_RULES.to_string()];

    if cfg.remove_fillers {
        sections.push(
            "Remove filler words and verbal tics: um, uh, er, hmm, \"you know\", \"I mean\", \
and repeated/stuttered words."
                .into(),
        );
    }
    if cfg.fix_punctuation {
        sections.push(
            "Add natural punctuation and capitalization: periods, commas, question marks. \
Split run-on sentences. Do not use em dashes unless the speaker asked for one."
                .into(),
        );
    }
    if cfg.fix_grammar {
        sections.push(
            "Fix obvious grammar slips and transcription artifacts, but keep the speaker's \
voice and word choice; do not paraphrase whole sentences."
                .into(),
        );
    }
    if cfg.format_lists {
        sections.push(
            "If the speaker enumerates items (\"first... second...\" or \"one... two...\"), \
format them as a list with each item on its own line."
                .into(),
        );
    }

    if let Some(p) = profile {
        let tone = match p.tone {
            Tone::Auto => None,
            Tone::Casual => Some("casual and conversational"),
            Tone::Professional => Some("professional and polished"),
            Tone::Technical => Some("technical and precise"),
        };
        let mut line = format!("The text will be inserted into {}.", p.name);
        if let Some(t) = tone {
            line.push_str(&format!(" Use a {t} tone."));
        }
        if !p.instructions.trim().is_empty() {
            line.push(' ');
            line.push_str(p.instructions.trim());
        }
        sections.push(line);
    }

    if !dictionary_terms.is_empty() {
        sections.push(format!(
            "Spell these user-specific terms exactly as written when they appear: {}.",
            dictionary_terms.join(", ")
        ));
    }

    if !cfg.custom_instructions.trim().is_empty() {
        sections.push(cfg.custom_instructions.trim().to_string());
    }

    sections.join("\n\n")
}

/// The user-role message wrapping the raw transcript.
pub fn build_user_prompt(transcript: &str) -> String {
    format!("Transcript:\n{transcript}")
}

/// System prompt for Command Mode: rewrite selected text per a spoken instruction.
pub const COMMAND_RULES: &str = "You are a text editor that applies a spoken instruction to a \
piece of text. Apply the instruction faithfully and return ONLY the edited text — no preamble, \
no explanation, no quotes or code fences. If the instruction is a question about the text rather \
than an edit, answer it concisely as plain text. Preserve the original formatting (line breaks, \
lists) unless the instruction says otherwise.";

/// User prompt for Command Mode.
pub fn build_command_prompt(instruction: &str, selected_text: &str) -> String {
    format!("Instruction: {instruction}\n\nText:\n{selected_text}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AppProfile;

    #[test]
    fn sections_toggle() {
        let mut cfg = CleanupSettings::default();
        cfg.format_lists = false;
        let p = build_system_prompt(&cfg, None, &[]);
        assert!(p.contains("filler words"));
        assert!(!p.contains("format them as a list"));
    }

    #[test]
    fn profile_and_dictionary_are_included() {
        let cfg = CleanupSettings::default();
        let profile = AppProfile {
            name: "Slack".into(),
            app_match: "slack".into(),
            tone: Tone::Casual,
            instructions: "Keep it brief.".into(),
            enabled: true,
        };
        let p = build_system_prompt(&cfg, Some(&profile), &["Kubernetes".into()]);
        assert!(p.contains("inserted into Slack"));
        assert!(p.contains("casual and conversational"));
        assert!(p.contains("Keep it brief."));
        assert!(p.contains("Kubernetes"));
    }
}
