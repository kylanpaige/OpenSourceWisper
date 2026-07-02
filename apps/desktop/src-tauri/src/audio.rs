//! Microphone capture via cpal.
//!
//! cpal streams are not `Send` on every platform, so the stream lives on its
//! own thread; samples accumulate in a shared buffer at the device's native
//! rate and are downmixed/resampled to 16 kHz mono (what whisper.cpp expects)
//! when the recording stops.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

pub struct Recorder {
    stop_flag: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    samples: Arc<Mutex<Vec<f32>>>,
    source_rate: Arc<Mutex<u32>>,
    started: Instant,
    /// Rolling ms of trailing silence, updated by the level meter.
    pub last_voice: Arc<Mutex<Instant>>,
}

pub struct Recording {
    /// 16 kHz mono samples ready for whisper.
    pub samples: Vec<f32>,
    pub duration_ms: u64,
}

impl Recorder {
    /// Start capturing from `device_name` (empty = default input device).
    pub fn start(
        app: AppHandle,
        device_name: &str,
        silence_threshold: f32,
    ) -> Result<Self, String> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let source_rate = Arc::new(Mutex::new(WHISPER_SAMPLE_RATE));
        let last_voice = Arc::new(Mutex::new(Instant::now()));

        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let device_name = device_name.to_string();
        let thread = {
            let stop_flag = stop_flag.clone();
            let samples = samples.clone();
            let source_rate = source_rate.clone();
            let last_voice = last_voice.clone();
            std::thread::spawn(move || {
                let result = run_stream(
                    app,
                    &device_name,
                    silence_threshold,
                    stop_flag,
                    samples,
                    source_rate,
                    last_voice,
                    ready_tx.clone(),
                );
                if let Err(e) = result {
                    // Surface setup errors through the ready channel if still awaited.
                    let _ = ready_tx.send(Err(e));
                }
            })
        };

        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Self {
                stop_flag,
                thread: Some(thread),
                samples,
                source_rate,
                started: Instant::now(),
                last_voice,
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("timed out opening the microphone".into()),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    pub fn silence_for(&self) -> Duration {
        self.last_voice.lock().unwrap().elapsed()
    }

    /// Stop the stream and return 16 kHz mono audio.
    pub fn stop(mut self) -> Recording {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        let raw = std::mem::take(&mut *self.samples.lock().unwrap());
        let rate = *self.source_rate.lock().unwrap();
        let duration_ms = (raw.len() as u64 * 1000) / rate.max(1) as u64;
        Recording {
            samples: resample_to_16k(&raw, rate),
            duration_ms,
        }
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_stream(
    app: AppHandle,
    device_name: &str,
    silence_threshold: f32,
    stop_flag: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    source_rate: Arc<Mutex<u32>>,
    last_voice: Arc<Mutex<Instant>>,
    ready_tx: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let host = cpal::default_host();
    let device = if device_name.is_empty() {
        host.default_input_device()
    } else {
        host.input_devices()
            .map_err(|e| e.to_string())?
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .or_else(|| host.default_input_device())
    }
    .ok_or("no microphone found")?;

    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let channels = config.channels() as usize;
    let rate = config.sample_rate().0;
    *source_rate.lock().unwrap() = rate;

    // Downmix to mono in the callback; accumulate into the shared buffer.
    // A level event fires every ~60 ms for the UI meter and silence tracking.
    let mut meter_buf: Vec<f32> = Vec::with_capacity(2048);
    let mut last_emit = Instant::now();
    let data_cb = move |data: &[f32]| {
        let mut mono: Vec<f32> = Vec::with_capacity(data.len() / channels);
        for frame in data.chunks_exact(channels) {
            mono.push(frame.iter().sum::<f32>() / channels as f32);
        }
        meter_buf.extend_from_slice(&mono);
        samples.lock().unwrap().extend_from_slice(&mono);
        if last_emit.elapsed() >= Duration::from_millis(60) && !meter_buf.is_empty() {
            let rms = (meter_buf.iter().map(|s| s * s).sum::<f32>() / meter_buf.len() as f32)
                .sqrt();
            meter_buf.clear();
            last_emit = Instant::now();
            if rms > silence_threshold {
                *last_voice.lock().unwrap() = Instant::now();
            }
            let _ = app.emit("audio-level", rms);
        }
    };

    let err_cb = |e: cpal::StreamError| log::error!("audio stream error: {e}");
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let mut cb = data_cb;
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| cb(data),
                err_cb,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let mut cb = data_cb;
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|s| *s as f32 / i16::MAX as f32).collect();
                    cb(&f);
                },
                err_cb,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let mut cb = data_cb;
            device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    let f: Vec<f32> = data
                        .iter()
                        .map(|s| (*s as f32 - u16::MAX as f32 / 2.0) / (u16::MAX as f32 / 2.0))
                        .collect();
                    cb(&f);
                },
                err_cb,
                None,
            )
        }
        other => {
            return Err(format!("unsupported sample format: {other:?}"));
        }
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    let _ = ready_tx.send(Ok(()));

    while !stop_flag.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(stream);
    Ok(())
}

/// Linear-interpolation resample to 16 kHz. Speech tolerates this well and it
/// avoids pulling in a DSP dependency; swap for rubato if quality complaints appear.
fn resample_to_16k(input: &[f32], from_rate: u32) -> Vec<f32> {
    if from_rate == WHISPER_SAMPLE_RATE || input.is_empty() {
        return input.to_vec();
    }
    let ratio = from_rate as f64 / WHISPER_SAMPLE_RATE as f64;
    let out_len = (input.len() as f64 / ratio).floor() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos as usize;
        let frac = (pos - idx as f64) as f32;
        let a = input[idx.min(input.len() - 1)];
        let b = input[(idx + 1).min(input.len() - 1)];
        out.push(a + (b - a) * frac);
    }
    out
}

/// Names of available input devices, for the settings UI.
pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devs| devs.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}
