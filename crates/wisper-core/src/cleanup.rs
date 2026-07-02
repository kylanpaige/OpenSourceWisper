//! Local-LLM cleanup client: Ollama native API or any OpenAI-compatible server.
//!
//! Fail-open by design: any error (server down, timeout, bad response) returns
//! `CleanupOutcome::Skipped` with a reason, and the caller injects the
//! deterministically-formatted transcript instead. Dictation must never block
//! on a flaky LLM.

use crate::prompts::{build_system_prompt, build_user_prompt};
use crate::settings::{AppProfile, CleanupProvider, CleanupSettings};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum CleanupOutcome {
    Cleaned(String),
    /// Cleanup didn't run or failed; the reason is surfaced in the UI/log.
    Skipped(String),
}

impl CleanupOutcome {
    pub fn text_or<'a>(&'a self, fallback: &'a str) -> &'a str {
        match self {
            CleanupOutcome::Cleaned(t) => t,
            CleanupOutcome::Skipped(_) => fallback,
        }
    }
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaMessage>,
}

#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OllamaMessage,
}

pub async fn clean_transcript(
    cfg: &CleanupSettings,
    transcript: &str,
    profile: Option<&AppProfile>,
    dictionary_terms: &[String],
) -> CleanupOutcome {
    if !cfg.enabled {
        return CleanupOutcome::Skipped("cleanup disabled".into());
    }
    let transcript = transcript.trim();
    if transcript.chars().count() < cfg.min_chars {
        return CleanupOutcome::Skipped(format!(
            "transcript under {} chars; skipping LLM",
            cfg.min_chars
        ));
    }

    let system = build_system_prompt(cfg, profile, dictionary_terms);
    let user = build_user_prompt(transcript);

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(cfg.timeout_ms.max(1000)))
        .build()
    {
        Ok(c) => c,
        Err(e) => return CleanupOutcome::Skipped(format!("http client error: {e}")),
    };

    let base = cfg.base_url.trim_end_matches('/');
    let result = match cfg.provider {
        CleanupProvider::Ollama => {
            let body = json!({
                "model": cfg.model,
                "stream": false,
                "options": { "temperature": 0.2 },
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user },
                ],
            });
            let resp = client.post(format!("{base}/api/chat")).json(&body).send().await;
            match resp {
                Ok(r) if r.status().is_success() => r
                    .json::<OllamaChatResponse>()
                    .await
                    .ok()
                    .and_then(|r| r.message)
                    .map(|m| m.content),
                Ok(r) => return CleanupOutcome::Skipped(format!("ollama returned {}", r.status())),
                Err(e) => return CleanupOutcome::Skipped(format!("ollama unreachable: {e}")),
            }
        }
        CleanupProvider::OpenAiCompat => {
            let body = json!({
                "model": cfg.model,
                "temperature": 0.2,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user },
                ],
            });
            let resp = client
                .post(format!("{base}/v1/chat/completions"))
                .json(&body)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => r
                    .json::<OpenAiChatResponse>()
                    .await
                    .ok()
                    .and_then(|r| r.choices.into_iter().next())
                    .map(|c| c.message.content),
                Ok(r) => return CleanupOutcome::Skipped(format!("server returned {}", r.status())),
                Err(e) => return CleanupOutcome::Skipped(format!("server unreachable: {e}")),
            }
        }
    };

    match result {
        Some(text) => {
            let cleaned = sanitize_llm_output(&text, transcript);
            if cleaned.is_empty() {
                CleanupOutcome::Skipped("LLM returned empty text".into())
            } else {
                CleanupOutcome::Cleaned(cleaned)
            }
        }
        None => CleanupOutcome::Skipped("malformed LLM response".into()),
    }
}

/// Command Mode: apply a spoken instruction to selected text via the local LLM.
/// Unlike `clean_transcript`, this requires the LLM — there is no deterministic
/// fallback for "make this more formal".
pub async fn command_edit(
    cfg: &CleanupSettings,
    instruction: &str,
    selected_text: &str,
) -> Result<String, String> {
    let system = crate::prompts::COMMAND_RULES.to_string();
    let user = crate::prompts::build_command_prompt(instruction.trim(), selected_text);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(cfg.timeout_ms.max(1000) * 2))
        .build()
        .map_err(|e| format!("http client error: {e}"))?;

    let base = cfg.base_url.trim_end_matches('/');
    let content = match cfg.provider {
        CleanupProvider::Ollama => {
            let body = json!({
                "model": cfg.model,
                "stream": false,
                "options": { "temperature": 0.3 },
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user },
                ],
            });
            let r = client
                .post(format!("{base}/api/chat"))
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("LLM unreachable: {e}"))?;
            if !r.status().is_success() {
                return Err(format!("LLM returned {}", r.status()));
            }
            r.json::<OllamaChatResponse>()
                .await
                .ok()
                .and_then(|r| r.message)
                .map(|m| m.content)
        }
        CleanupProvider::OpenAiCompat => {
            let body = json!({
                "model": cfg.model,
                "temperature": 0.3,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": user },
                ],
            });
            let r = client
                .post(format!("{base}/v1/chat/completions"))
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("LLM unreachable: {e}"))?;
            if !r.status().is_success() {
                return Err(format!("LLM returned {}", r.status()));
            }
            r.json::<OpenAiChatResponse>()
                .await
                .ok()
                .and_then(|r| r.choices.into_iter().next())
                .map(|c| c.message.content)
        }
    };

    let content = content.ok_or("malformed LLM response")?;
    let mut text = content.trim();
    if text.starts_with("```") {
        text = text.trim_start_matches("```").trim_start_matches(|c| c != '\n');
        text = text.trim_end_matches("```");
    }
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("LLM returned empty text".into());
    }
    Ok(text)
}

/// Guard against common small-model misbehavior: preambles, wrapping quotes,
/// code fences, and hallucinated essays. If the output looks unusable, fall
/// back to an empty string so the caller keeps the raw transcript.
pub fn sanitize_llm_output(output: &str, transcript: &str) -> String {
    let mut text = output.trim();

    // Strip code fences.
    if text.starts_with("```") {
        text = text.trim_start_matches("```").trim_start_matches(|c| c != '\n');
        text = text.trim_end_matches("```");
    }
    let mut text = text.trim().to_string();

    // Strip a leading "Cleaned text:"/"Here is..." preamble line if present.
    if let Some(idx) = text.find('\n') {
        let first_line = text[..idx].to_lowercase();
        if (first_line.starts_with("here") || first_line.ends_with(':'))
            && first_line.len() < 80
            && (first_line.contains("clean") || first_line.contains("text") || first_line.contains("transcript"))
        {
            text = text[idx + 1..].trim().to_string();
        }
    }

    // Strip symmetrical wrapping quotes.
    if text.len() >= 2 {
        let bytes = text.as_bytes();
        if (bytes[0] == b'"' && bytes[text.len() - 1] == b'"')
            || (text.starts_with('\u{201C}') && text.ends_with('\u{201D}'))
    {
            text = text[1..].trim_end_matches(['"', '\u{201D}']).trim().to_string();
        }
    }

    // Reject wild expansions: cleanup should shorten or roughly preserve length.
    let t_len = transcript.chars().count().max(1);
    if text.chars().count() > t_len * 3 + 80 {
        return String::new();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_or_short_is_skipped() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut cfg = CleanupSettings::default();
        cfg.enabled = false;
        let out = rt.block_on(clean_transcript(&cfg, "hello there everyone", None, &[]));
        assert!(matches!(out, CleanupOutcome::Skipped(_)));

        cfg.enabled = true;
        cfg.min_chars = 100;
        let out = rt.block_on(clean_transcript(&cfg, "short", None, &[]));
        assert!(matches!(out, CleanupOutcome::Skipped(_)));
    }

    #[test]
    fn unreachable_server_fails_open() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut cfg = CleanupSettings::default();
        cfg.enabled = true;
        cfg.min_chars = 1;
        cfg.timeout_ms = 1500;
        cfg.base_url = "http://127.0.0.1:1".into(); // nothing listens here
        let out = rt.block_on(clean_transcript(&cfg, "hello world this is a test", None, &[]));
        assert!(matches!(out, CleanupOutcome::Skipped(_)));
    }

    #[test]
    fn sanitize_strips_fences_quotes_and_preambles() {
        let t = "send the report by monday";
        assert_eq!(
            sanitize_llm_output("```\nSend the report by Monday.\n```", t),
            "Send the report by Monday."
        );
        assert_eq!(
            sanitize_llm_output("\"Send the report by Monday.\"", t),
            "Send the report by Monday."
        );
        assert_eq!(
            sanitize_llm_output("Here is the cleaned text:\nSend the report by Monday.", t),
            "Send the report by Monday."
        );
    }

    #[test]
    fn sanitize_rejects_hallucinated_essays() {
        let out = sanitize_llm_output(&"blah ".repeat(200), "short input");
        assert!(out.is_empty());
    }

    #[test]
    fn outcome_text_or_falls_back() {
        let c = CleanupOutcome::Cleaned("clean".into());
        assert_eq!(c.text_or("raw"), "clean");
        let s = CleanupOutcome::Skipped("why".into());
        assert_eq!(s.text_or("raw"), "raw");
    }
}
