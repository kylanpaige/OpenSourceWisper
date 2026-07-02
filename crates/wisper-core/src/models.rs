//! Catalog of downloadable whisper.cpp (GGML) models.
//!
//! All URLs point at ggerganov/whisper.cpp's official converted checkpoints on
//! Hugging Face. Sizes are approximate download sizes used for progress UI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// On-disk filename, e.g. "ggml-base.en.bin". Also the settings key.
    pub file: String,
    pub label: String,
    pub description: String,
    pub size_mb: u64,
    /// English-only models are faster/more accurate for English speech.
    pub english_only: bool,
    pub recommended: bool,
    pub url: String,
}

const HF_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

fn model(
    file: &str,
    label: &str,
    description: &str,
    size_mb: u64,
    english_only: bool,
    recommended: bool,
) -> ModelInfo {
    ModelInfo {
        file: file.into(),
        label: label.into(),
        description: description.into(),
        size_mb,
        english_only,
        recommended,
        url: format!("{HF_BASE}/{file}"),
    }
}

pub fn catalog() -> Vec<ModelInfo> {
    vec![
        model(
            "ggml-tiny.en.bin",
            "Tiny (English)",
            "Fastest, lowest accuracy. Good for older CPUs or instant partial results.",
            75,
            true,
            false,
        ),
        model(
            "ggml-tiny.bin",
            "Tiny (Multilingual)",
            "Fastest multilingual option.",
            75,
            false,
            false,
        ),
        model(
            "ggml-base.en.bin",
            "Base (English)",
            "Great speed/accuracy balance for English dictation on any modern machine.",
            142,
            true,
            true,
        ),
        model(
            "ggml-base.bin",
            "Base (Multilingual)",
            "Balanced multilingual model.",
            142,
            false,
            false,
        ),
        model(
            "ggml-small.en.bin",
            "Small (English)",
            "Noticeably better accuracy; still real-time on Apple Silicon and recent CPUs.",
            466,
            true,
            true,
        ),
        model(
            "ggml-small.bin",
            "Small (Multilingual)",
            "Better multilingual accuracy.",
            466,
            false,
            false,
        ),
        model(
            "ggml-medium.en.bin",
            "Medium (English)",
            "High accuracy; needs a fast CPU or GPU to stay snappy.",
            1500,
            true,
            false,
        ),
        model(
            "ggml-large-v3-turbo.bin",
            "Large v3 Turbo",
            "Best accuracy, robust to noise and accents. Recommended on Apple Silicon / NVIDIA GPU.",
            1620,
            false,
            true,
        ),
        model(
            "ggml-large-v3-turbo-q5_0.bin",
            "Large v3 Turbo (Q5 quantized)",
            "Near-turbo accuracy at roughly a third of the size and memory.",
            574,
            false,
            false,
        ),
    ]
}

pub fn find(file: &str) -> Option<ModelInfo> {
    catalog().into_iter().find(|m| m.file == file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_sane() {
        let all = catalog();
        assert!(all.len() >= 6);
        assert!(all.iter().any(|m| m.recommended));
        for m in &all {
            assert!(m.url.starts_with("https://huggingface.co/"));
            assert!(m.url.ends_with(&m.file));
            assert!(m.size_mb > 0);
        }
    }

    #[test]
    fn find_by_file() {
        assert!(find("ggml-base.en.bin").is_some());
        assert!(find("nope.bin").is_none());
    }
}
