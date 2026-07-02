//! The dictation pipeline: record → transcribe → dictionary → format → LLM
//! cleanup → inject → history. Orchestrated off the main thread; progress is
//! mirrored to the UI via events.

use crate::asr::{dictionary_prompt, WhisperEngine};
use crate::audio::Recorder;
use crate::chime::{self, Chime};
use crate::state::{AppState, DictationPhase, RecordingPurpose};
use crate::{context, inject, paths};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use wisper_core::cleanup::{clean_transcript, command_edit, CleanupOutcome};
use wisper_core::settings::RecordingMode;
use wisper_core::{dictionary, format, history};

#[derive(Debug, Clone, Serialize)]
pub struct DictationResult {
    pub raw: String,
    pub final_text: String,
    pub app: String,
    pub duration_ms: u64,
    pub cleaned: bool,
    /// Why the LLM stage was skipped, if it was.
    pub skip_reason: String,
}

fn set_phase(app: &AppHandle, phase: DictationPhase) {
    let state = app.state::<AppState>();
    *state.phase.lock().unwrap() = phase;
    let _ = app.emit("dictation-phase", phase);
    crate::overlay::sync(app, phase);

    // Keep the tray tooltip honest about what the app is doing.
    if let Some(tray) = app.tray_by_id("main-tray") {
        let tip = match phase {
            DictationPhase::Idle => "OpenSourceWisper — local dictation".to_string(),
            DictationPhase::Recording => "OpenSourceWisper — listening…".to_string(),
            DictationPhase::Transcribing => "OpenSourceWisper — transcribing…".to_string(),
            DictationPhase::Cleaning => "OpenSourceWisper — polishing…".to_string(),
            DictationPhase::Injecting => "OpenSourceWisper — inserting…".to_string(),
        };
        let _ = tray.set_tooltip(Some(tip));
    }
}

fn fail(app: &AppHandle, msg: String) {
    log::error!("{msg}");
    let _ = app.emit("dictation-error", msg);
    let state = app.state::<AppState>();
    state.busy.store(false, Ordering::SeqCst);
    *state.purpose.lock().unwrap() = RecordingPurpose::Dictate;
    set_phase(app, DictationPhase::Idle);
}

/// Begin recording (hotkey press or UI button). No-op if already recording/busy.
pub fn start_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.busy.load(Ordering::SeqCst) {
        return;
    }
    {
        let recorder = state.recorder.lock().unwrap();
        if recorder.is_some() {
            return;
        }
    }
    let (device, threshold, mode, autostop_ms, max_secs) = {
        let s = state.settings.lock().unwrap();
        (
            s.audio.input_device.clone(),
            s.audio.silence_threshold,
            s.recording_mode,
            s.audio.silence_autostop_ms,
            s.audio.max_recording_secs,
        )
    };
    let chimes = state.settings.lock().unwrap().audio.chimes;
    match Recorder::start(app.clone(), &device, threshold) {
        Ok(rec) => {
            *state.recorder.lock().unwrap() = Some(rec);
            set_phase(app, DictationPhase::Recording);
            if chimes {
                chime::play(Chime::Start);
            }
            spawn_watchdog(app.clone(), mode, autostop_ms, max_secs);
        }
        Err(e) => fail(app, format!("could not start recording: {e}")),
    }
}

/// Command Mode: capture the current selection, then record the spoken
/// instruction. On release, the selection is rewritten by the local LLM.
pub fn start_command(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.busy.load(Ordering::SeqCst) || state.recorder.lock().unwrap().is_some() {
        return;
    }
    if !state.settings.lock().unwrap().cleanup.enabled {
        let _ = app.emit(
            "dictation-error",
            "Command Mode needs AI cleanup enabled (Settings → AI Cleanup)".to_string(),
        );
        return;
    }
    let selection = match inject::copy_selection() {
        Ok(Some(s)) => s,
        Ok(None) => {
            let _ = app.emit(
                "dictation-error",
                "Command Mode: select some text first, then hold the hotkey and speak".to_string(),
            );
            return;
        }
        Err(e) => return fail(app, format!("could not read selection: {e}")),
    };
    *state.purpose.lock().unwrap() = RecordingPurpose::Command { selection };
    start_recording(app);
}

/// In toggle mode, auto-stop on prolonged silence; in every mode, enforce the
/// max recording duration so a stuck hotkey can't record forever.
fn spawn_watchdog(app: AppHandle, mode: RecordingMode, autostop_ms: u64, max_secs: u64) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            let state = app.state::<AppState>();
            let (elapsed, silent) = {
                let guard = state.recorder.lock().unwrap();
                match guard.as_ref() {
                    Some(r) => (r.elapsed(), r.silence_for()),
                    None => return, // recording ended
                }
            };
            let over_max = max_secs > 0 && elapsed.as_secs() >= max_secs;
            let auto_silence = mode == RecordingMode::Toggle
                && autostop_ms > 0
                && elapsed.as_millis() > 1500
                && silent.as_millis() as u64 >= autostop_ms;
            if over_max || auto_silence {
                stop_and_process(&app);
                return;
            }
        }
    });
}

/// Stop recording and run the rest of the pipeline (hotkey release, toggle
/// press, silence auto-stop, or UI button).
pub fn stop_and_process(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Some(recorder) = state.recorder.lock().unwrap().take() else {
        return;
    };
    if state.busy.swap(true, Ordering::SeqCst) {
        return;
    }
    let purpose = std::mem::take(&mut *state.purpose.lock().unwrap());
    if state.settings.lock().unwrap().audio.chimes {
        chime::play(Chime::Stop);
    }
    // Capture the focused app BEFORE transcription so slow ASR can't misattribute.
    let focused_app = context::focused_app_name();
    set_phase(app, DictationPhase::Transcribing);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let recording = tokio::task::spawn_blocking(move || recorder.stop())
            .await
            .expect("recorder stop panicked");

        if recording.samples.len() < 1600 {
            // Under ~100ms of audio: accidental tap; drop it quietly.
            let state = app.state::<AppState>();
            state.busy.store(false, Ordering::SeqCst);
            set_phase(&app, DictationPhase::Idle);
            return;
        }

        // Snapshot everything the pipeline needs, then release locks.
        let (settings, dict_entries) = {
            let state = app.state::<AppState>();
            let s = state.settings.lock().unwrap().clone();
            let d = state.dictionary.lock().unwrap().clone();
            (s, d)
        };

        // --- Stage 1: ASR (blocking, CPU/GPU heavy) ---
        let model_file = settings.model.clone();
        let model_path = paths::models_dir(&app).join(&model_file);
        let language = settings.language.clone();
        let threads = settings.threads;
        let prompt_terms = dictionary::terms_for_prompt(&dict_entries, 24);
        let initial_prompt = dictionary_prompt(&prompt_terms);
        let samples = recording.samples;

        let app2 = app.clone();
        let raw = tokio::task::spawn_blocking(move || {
            let state = app2.state::<AppState>();
            let mut engine_slot = state.engine.lock().unwrap();
            let needs_load = engine_slot
                .as_ref()
                .map(|e| e.model_file != model_file)
                .unwrap_or(true);
            if needs_load {
                *engine_slot = Some(WhisperEngine::load(&model_path, &model_file)?);
            }
            engine_slot
                .as_ref()
                .unwrap()
                .transcribe(&samples, &language, threads, &initial_prompt)
        })
        .await
        .unwrap_or_else(|e| Err(format!("transcription task panicked: {e}")));

        let raw = match raw {
            Ok(t) => t,
            Err(e) => return fail(&app, e),
        };
        if raw.is_empty() || raw.chars().all(|c| !c.is_alphanumeric()) {
            let state = app.state::<AppState>();
            state.busy.store(false, Ordering::SeqCst);
            set_phase(&app, DictationPhase::Idle);
            let _ = app.emit("dictation-error", "no speech detected".to_string());
            return;
        }

        // Command Mode: the transcript is an instruction, not content.
        if let RecordingPurpose::Command { selection } = purpose {
            set_phase(&app, DictationPhase::Cleaning);
            let edited = command_edit(&settings.cleanup, &raw, &selection).await;
            match edited {
                Ok(text) => {
                    set_phase(&app, DictationPhase::Injecting);
                    let injection = settings.injection;
                    let to_inject = text.clone();
                    let r = tokio::task::spawn_blocking(move || {
                        inject::inject_text(&to_inject, injection)
                    })
                    .await
                    .unwrap_or_else(|e| Err(format!("injection task panicked: {e}")));
                    if let Err(e) = r {
                        let _ = app.emit("dictation-error", format!("could not insert text: {e}"));
                    }
                    let _ = app.emit(
                        "dictation-result",
                        DictationResult {
                            raw: format!("[command] {raw}"),
                            final_text: text,
                            app: focused_app,
                            duration_ms: recording.duration_ms,
                            cleaned: true,
                            skip_reason: String::new(),
                        },
                    );
                }
                Err(e) => {
                    let _ = app.emit("dictation-error", format!("Command Mode failed: {e}"));
                }
            }
            let state = app.state::<AppState>();
            state.busy.store(false, Ordering::SeqCst);
            set_phase(&app, DictationPhase::Idle);
            return;
        }

        // --- Stage 2: dictionary + deterministic formatting ---
        let with_dict = dictionary::apply(&dict_entries, &raw);
        let formatted = format::post_process(&with_dict, &settings.format);

        // --- Stage 3: local LLM cleanup (fail-open) ---
        set_phase(&app, DictationPhase::Cleaning);
        let profile = settings.profile_for_app(&focused_app).cloned();
        let outcome = clean_transcript(
            &settings.cleanup,
            &formatted,
            profile.as_ref(),
            &dictionary::terms_for_prompt(&dict_entries, 24),
        )
        .await;
        let (final_text, cleaned, skip_reason) = match outcome {
            CleanupOutcome::Cleaned(t) => {
                // Dictionary once more: the LLM may reintroduce misspellings.
                (dictionary::apply(&dict_entries, &t), true, String::new())
            }
            CleanupOutcome::Skipped(reason) => (formatted.clone(), false, reason),
        };

        // --- Stage 4: inject into the focused app ---
        set_phase(&app, DictationPhase::Injecting);
        let injection = settings.injection;
        let text_to_inject = final_text.clone();
        let inject_result =
            tokio::task::spawn_blocking(move || inject::inject_text(&text_to_inject, injection))
                .await
                .unwrap_or_else(|e| Err(format!("injection task panicked: {e}")));
        if let Err(e) = inject_result {
            let _ = app.emit("dictation-error", format!("could not insert text: {e}"));
        }

        // --- Stage 5: history + result event ---
        let result = DictationResult {
            raw,
            final_text,
            app: focused_app,
            duration_ms: recording.duration_ms,
            cleaned,
            skip_reason,
        };
        if settings.save_history {
            let entry = history::HistoryEntry {
                timestamp: chrono::Utc::now(),
                raw: result.raw.clone(),
                final_text: result.final_text.clone(),
                app: result.app.clone(),
                duration_ms: result.duration_ms,
                cleaned: result.cleaned,
            };
            let path = paths::history_path(&app);
            if history::append(&path, &entry).is_ok() {
                // Occasionally trim the file to the configured cap.
                let _ = history::compact(&path, settings.max_history_entries);
            }
        }
        let _ = app.emit("dictation-result", &result);

        let state = app.state::<AppState>();
        state.busy.store(false, Ordering::SeqCst);
        set_phase(&app, DictationPhase::Idle);
    });
}

/// Discard the in-flight recording without transcribing (Esc / cancel).
pub fn cancel_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    let taken = state.recorder.lock().unwrap().take();
    *state.purpose.lock().unwrap() = RecordingPurpose::Dictate;
    if let Some(rec) = taken {
        let _ = rec.stop();
        set_phase(app, DictationPhase::Idle);
    }
}

/// Toggle used by the tray, the toggle hotkey, and toggle recording mode.
pub fn toggle_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    let is_recording = state.recorder.lock().unwrap().is_some();
    if is_recording {
        stop_and_process(app);
    } else {
        start_recording(app);
    }
}
