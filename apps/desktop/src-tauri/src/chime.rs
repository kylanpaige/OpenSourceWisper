//! Start/stop chimes synthesized directly through cpal — no audio assets,
//! no extra dependencies. Short, quiet, and skippable via settings.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::Duration;

pub enum Chime {
    Start,
    Stop,
}

/// Fire-and-forget: plays on its own thread, never blocks the pipeline.
pub fn play(chime: Chime) {
    std::thread::spawn(move || {
        let (freq, ms) = match chime {
            Chime::Start => (880.0_f32, 90),
            Chime::Stop => (523.25_f32, 90),
        };
        if let Err(e) = play_tone(freq, ms) {
            log::debug!("chime skipped: {e}");
        }
    });
}

fn play_tone(freq: f32, ms: u64) -> Result<(), String> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("no output device")?;
    let config = device.default_output_config().map_err(|e| e.to_string())?;
    if config.sample_format() != cpal::SampleFormat::F32 {
        return Err("non-f32 output".into()); // rare; not worth converting for a chime
    }
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;
    let total = (sample_rate * ms as f32 / 1000.0) as usize;
    let mut i = 0usize;

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                for frame in data.chunks_mut(channels) {
                    let t = i as f32 / sample_rate;
                    // Sine with a fast attack/decay envelope so it doesn't click.
                    let env = if i >= total {
                        0.0
                    } else {
                        let p = i as f32 / total as f32;
                        (1.0 - p).min(p * 8.0).clamp(0.0, 1.0)
                    };
                    let s = (t * freq * std::f32::consts::TAU).sin() * 0.12 * env;
                    for out in frame.iter_mut() {
                        *out = s;
                    }
                    i += 1;
                }
            },
            |e| log::debug!("chime stream error: {e}"),
            None,
        )
        .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_millis(ms + 60));
    Ok(())
}
