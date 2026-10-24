//! Built-in polyphonic synthesizer backend powered by Glicol and CPAL.

use std::collections::VecDeque;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use glicol::Engine;
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SynthError {
    #[error("No default audio output device available: {0}")]
    DeviceNotFound(String),
    #[error("Failed to query default output stream configuration: {0}")]
    DefaultConfigError(String),
    #[error("Failed to build CPAL audio output stream: {0}")]
    BuildStreamError(String),
    #[error("Failed to start CPAL audio stream: {0}")]
    PlayStreamError(String),
    #[error("Synth command ring buffer is full")]
    BufferFull,
}

#[derive(Debug, Clone)]
pub enum SynthCommand {
    Notes(Vec<u8>),
    Mute(bool),
    Volume(f32),
}

/// Generates a Glicol script representing an "init" polyphonic sine wave patch.
///
/// When `notes` is empty, outputs silence (`o: const_sig 0.0;`).
/// When notes are active, each note gets its own sine oscillator scaled by `1 / sqrt(N)`
/// to preserve perceived volume and prevent clipping, summed into the stereo output `o`.
#[must_use]
pub fn generate_init_preset(notes: &[u8], master_volume: f32) -> String {
    if notes.is_empty() {
        return "o: const_sig 0.0;".to_string();
    }

    let num_notes = notes.len().max(1) as f32;
    // Scale per-voice amplitude inversely with sqrt(N) to prevent clipping
    let voice_gain = (master_volume / num_notes.sqrt()).min(0.5);

    let mut code = String::with_capacity(notes.len() * 48 + 32);
    let mut voice_names = Vec::with_capacity(notes.len());

    for (i, &note) in notes.iter().enumerate() {
        // Equal temperament (12-TET) frequency calculation
        let freq = 440.0 * 2.0_f32.powf((f32::from(note) - 69.0) / 12.0);
        let v_name = format!("~v{i}");
        code.push_str(&format!("{v_name}: sin {freq:.2} >> mul {voice_gain:.4};\n"));
        voice_names.push(v_name);
    }

    if voice_names.len() == 1 {
        if let Some(v0) = voice_names.first() {
            code.push_str(&format!("o: {v0};\n"));
        }
    } else if let Some((first, rest)) = voice_names.split_first() {
        code.push_str(&format!("o: {first}"));
        for v in rest {
            code.push_str(&format!(" >> add {v}"));
        }
        code.push_str(";\n");
    }

    code
}

pub type SynthProd = <HeapRb<SynthCommand> as Split>::Prod;

pub struct SynthEngine {
    tx: SynthProd,
    muted: bool,
    volume: f32,
    device_name: String,
    _stream: cpal::Stream,
}

impl SynthEngine {
    /// Attempts to initialize the default audio output device and start the Glicol synth stream.
    pub fn new() -> Result<Self, SynthError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| SynthError::DeviceNotFound("No default output device".to_string()))?;

        let device_name = device
            .name()
            .unwrap_or_else(|_| "Default Audio Device".to_string());

        let default_config = device
            .default_output_config()
            .map_err(|e| SynthError::DefaultConfigError(e.to_string()))?;

        let sample_rate = default_config.sample_rate().0;
        let channels = default_config.channels() as usize;

        let rb = HeapRb::<SynthCommand>::new(64);
        let (tx, mut rx) = rb.split();

        let initial_volume = 0.35_f32;
        let mut engine = Engine::<128>::new();
        engine.set_sr(sample_rate as usize);
        engine.update_with_code("o: const_sig 0.0;");

        let mut fifo: VecDeque<(f32, f32)> = VecDeque::with_capacity(4096);
        let mut current_notes: Vec<u8> = Vec::new();
        let mut active_preset_notes: Vec<u8> = Vec::new();
        let mut muted = false;
        let mut volume = initial_volume;
        let mut smoothed_gain = 0.0_f32;

        let stream_config: cpal::StreamConfig = default_config.into();

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // 1. Process any incoming commands from the UI thread
                    while let Some(cmd) = rx.try_pop() {
                        match cmd {
                            SynthCommand::Notes(notes) => {
                                if current_notes != notes {
                                    current_notes = notes;
                                    if !current_notes.is_empty() && !muted {
                                        active_preset_notes = current_notes.clone();
                                        let code = generate_init_preset(&active_preset_notes, volume);
                                        engine.update_with_code(&code);
                                    }
                                }
                            }
                            SynthCommand::Mute(new_muted) => {
                                if muted != new_muted {
                                    muted = new_muted;
                                    if !muted && !current_notes.is_empty() {
                                        active_preset_notes = current_notes.clone();
                                        let code = generate_init_preset(&active_preset_notes, volume);
                                        engine.update_with_code(&code);
                                    }
                                }
                            }
                            SynthCommand::Volume(new_vol) => {
                                volume = new_vol;
                                if !muted && !active_preset_notes.is_empty() {
                                    let code = generate_init_preset(&active_preset_notes, volume);
                                    engine.update_with_code(&code);
                                }
                            }
                        }
                    }

                    // 2. Calculate frames requested by CPAL
                    let frames_needed = data.len().checked_div(channels).unwrap_or(0);

                    // 3. Fill FIFO buffer from Glicol engine blocks of 128 samples
                    while fifo.len() < frames_needed {
                        let (buffers, _) = engine.next_block(vec![]);
                        for k in 0..128 {
                            let left = buffers.first().and_then(|b| b.get(k)).copied().unwrap_or(0.0);
                            let right = buffers.get(1).and_then(|b| b.get(k)).copied().unwrap_or(left);
                            fifo.push_back((left, right));
                        }
                    }

                    // 4. Write FIFO audio frames into CPAL interleaved buffer with smooth envelope
                    let target_gain = if muted || current_notes.is_empty() { 0.0_f32 } else { 1.0_f32 };

                    for i in 0..frames_needed {
                        let (raw_l, raw_r) = fifo.pop_front().unwrap_or((0.0, 0.0));

                        // Hexboard parity: 5ms attack, 250ms acoustic release tail ("damper fall")
                        let coef = if target_gain > smoothed_gain { 0.008_f32 } else { 0.00035_f32 };
                        smoothed_gain += (target_gain - smoothed_gain) * coef;

                        if smoothed_gain < 0.0005 && (current_notes.is_empty() || muted) && !active_preset_notes.is_empty() {
                            engine.update_with_code("o: const_sig 0.0;");
                            active_preset_notes.clear();
                            smoothed_gain = 0.0;
                        }

                        let l = raw_l * smoothed_gain;
                        let r = raw_r * smoothed_gain;

                        if channels == 2 {
                            if let Some(s0) = data.get_mut(i * 2) {
                                *s0 = l;
                            }
                            if let Some(s1) = data.get_mut(i * 2 + 1) {
                                *s1 = r;
                            }
                        } else if channels == 1 {
                            if let Some(s0) = data.get_mut(i) {
                                *s0 = (l + r) * 0.5;
                            }
                        } else {
                            if let Some(s0) = data.get_mut(i * channels) {
                                *s0 = l;
                            }
                            if let Some(s1) = data.get_mut(i * channels + 1) {
                                *s1 = r;
                            }
                            for c in 2..channels {
                                if let Some(sc) = data.get_mut(i * channels + c) {
                                    *sc = 0.0;
                                }
                            }
                        }
                    }
                },
                |err| {
                    log::error!("CPAL audio stream error: {err}");
                },
                None,
            )
            .map_err(|e| SynthError::BuildStreamError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| SynthError::PlayStreamError(e.to_string()))?;

        Ok(Self {
            tx,
            muted,
            volume: initial_volume,
            device_name,
            _stream: stream,
        })
    }

    /// Sends the current list of active MIDI notes to the synth.
    pub fn set_notes(&mut self, notes: Vec<u8>) -> Result<(), SynthError> {
        self.tx
            .try_push(SynthCommand::Notes(notes))
            .map_err(|_| SynthError::BufferFull)
    }

    /// Sets the synth mute state.
    pub fn set_muted(&mut self, muted: bool) -> Result<(), SynthError> {
        self.muted = muted;
        self.tx
            .try_push(SynthCommand::Mute(muted))
            .map_err(|_| SynthError::BufferFull)
    }

    /// Toggles the synth mute state, returning the new mute state.
    pub fn toggle_mute(&mut self) -> Result<bool, SynthError> {
        let new_state = !self.muted;
        self.set_muted(new_state)?;
        Ok(new_state)
    }

    #[must_use]
    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    #[must_use]
    #[allow(dead_code)]
    pub const fn volume(&self) -> f32 {
        self.volume
    }

    /// Adjusts master volume clamped between 0.0 and 1.0.
    pub fn set_volume(&mut self, volume: f32) -> Result<(), SynthError> {
        self.volume = volume.clamp(0.0, 1.0);
        self.tx
            .try_push(SynthCommand::Volume(self.volume))
            .map_err(|_| SynthError::BufferFull)
    }

    #[must_use]
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    #[must_use]
    pub const fn preset_name(&self) -> &'static str {
        "Sine Init"
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_notes_generates_silence() {
        let code = generate_init_preset(&[], 0.35);
        assert_eq!(code, "o: const_sig 0.0;");

        let mut engine = Engine::<128>::new();
        engine.update_with_code(&code);
        let (buffers, _) = engine.next_block(vec![]);
        let sum: f32 = buffers[0].iter().sum();
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn test_single_note_generates_valid_glicol() {
        let code = generate_init_preset(&[69], 0.35); // A4 = 440 Hz
        assert!(code.contains("sin 440.00"));
        assert!(code.contains("o: ~v0;"));

        let mut engine = Engine::<128>::new();
        engine.update_with_code(&code);
        let (buffers, _) = engine.next_block(vec![]);
        assert_eq!(buffers.len(), 2);
        assert_eq!(buffers[0].len(), 128);
    }

    #[test]
    fn test_octave_frequencies() {
        let code_a3 = generate_init_preset(&[57], 0.35);
        assert!(code_a3.contains("sin 220.00"));

        let code_a4 = generate_init_preset(&[69], 0.35);
        assert!(code_a4.contains("sin 440.00"));

        let code_a5 = generate_init_preset(&[81], 0.35);
        assert!(code_a5.contains("sin 880.00"));
    }

    #[test]
    fn test_large_voicing_gain_scaling() {
        let dense_voicing = vec![48, 52, 55, 59, 62, 65, 69, 72]; // 8 voices
        let code = generate_init_preset(&dense_voicing, 0.40);
        let mut engine = Engine::<128>::new();
        engine.update_with_code(&code);
        let (buffers, _) = engine.next_block(vec![]);
        for &sample in buffers[0].iter() {
            assert!(sample.abs() <= 1.0, "Sample should not clip: {sample}");
        }
    }
}
