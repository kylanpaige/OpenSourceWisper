//! Whisper transcription via whisper-rs (whisper.cpp bindings).
//!
//! The context (weights) is expensive to load, so it stays resident in
//! AppState and is only rebuilt when the selected model file changes.
//! Metal acceleration is enabled on macOS at compile time; CUDA behind the
//! `cuda` feature flag.

use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperEngine {
    ctx: WhisperContext,
    /// Model filename this context was loaded from (settings key).
    pub model_file: String,
}

impl WhisperEngine {
    pub fn load(model_path: &Path, model_file: &str) -> Result<Self, String> {
        if !model_path.exists() {
            return Err(format!(
                "model {} is not downloaded yet",
                model_path.display()
            ));
        }
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or("invalid model path")?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("failed to load whisper model: {e}"))?;
        Ok(Self {
            ctx,
            model_file: model_file.to_string(),
        })
    }

    /// Transcribe 16 kHz mono samples. `language` is "auto" or ISO 639-1.
    /// `initial_prompt` biases decoding toward user dictionary terms.
    pub fn transcribe(
        &self,
        samples: &[f32],
        language: &str,
        threads: usize,
        initial_prompt: &str,
    ) -> Result<String, String> {
        // whisper.cpp needs at least ~1s of audio to behave; pad short clips.
        let mut audio = samples.to_vec();
        let min_len = crate::audio::WHISPER_SAMPLE_RATE as usize;
        if audio.len() < min_len {
            audio.resize(min_len, 0.0);
        }

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("whisper state error: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        let lang = if language.is_empty() || language == "auto" {
            None
        } else {
            Some(language)
        };
        params.set_language(lang);
        let n_threads = if threads == 0 {
            (std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4))
            .min(8) as i32
        } else {
            threads as i32
        };
        params.set_n_threads(n_threads);
        if !initial_prompt.is_empty() {
            params.set_initial_prompt(initial_prompt);
        }
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_token_timestamps(false);
        params.set_translate(false);

        state
            .full(params, &audio)
            .map_err(|e| format!("transcription failed: {e}"))?;

        let n_segments = state
            .full_n_segments()
            .map_err(|e| format!("segment error: {e}"))?;
        let mut text = String::new();
        for i in 0..n_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }
        Ok(collapse_whitespace(text.trim()))
    }
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out
}

/// Build the initial prompt from dictionary terms: whisper biases toward
/// vocabulary it has "already seen" in the prompt.
pub fn dictionary_prompt(terms: &[String]) -> String {
    if terms.is_empty() {
        return String::new();
    }
    format!("Glossary: {}.", terms.join(", "))
}
