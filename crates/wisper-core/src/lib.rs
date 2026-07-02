//! Platform-agnostic core logic for OpenSourceWisper.
//!
//! Everything in this crate is pure logic or plain HTTP and is unit-testable
//! without an OS windowing system, microphone, or GPU. The desktop app
//! (apps/desktop) wires these pieces to cpal/whisper-rs/enigo/Tauri.

pub mod cleanup;
pub mod dictionary;
pub mod format;
pub mod history;
pub mod models;
pub mod prompts;
pub mod settings;

pub use settings::AppSettings;
