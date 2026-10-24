//! Built-in polyphonic synthesizer backend powered by CPAL with sample-accurate DSP.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use thiserror::Error;

pub const MAX_VOICES: usize = 16;

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

#[derive(Debug, Clone, Copy)]
pub enum SynthCommand {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    AllNotesOff,
    Mute(bool),
    Volume(f32),
}

#[derive(Debug, Clone, Copy, Default)]
struct Voice {
    note: u8,
    freq: f32,
    phase: f32,
    gain: f32,
    target_gain: f32,
    active: bool,
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
    /// Attempts to initialize the default audio output device and start the polyphonic synth stream.
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

        let sample_rate = default_config.sample_rate().0 as f32;
        let channels = default_config.channels() as usize;

        let rb = HeapRb::<SynthCommand>::new(128);
        let (tx, mut rx) = rb.split();

        let initial_volume = 0.50_f32;
        let mut voices = [Voice::default(); MAX_VOICES];
        let mut muted = false;
        let mut volume = initial_volume;

        let stream_config: cpal::StreamConfig = default_config.into();

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // 1. Process discrete note on / note off / volume commands
                    while let Some(cmd) = rx.try_pop() {
                        match cmd {
                            SynthCommand::NoteOn { note, velocity } => {
                                // 12-TET frequency formula: f = 440 * 2^((note - 69)/12)
                                let freq = 440.0 * 2.0_f32.powf((f32::from(note) - 69.0) / 12.0);
                                let vel_gain = (f32::from(velocity) / 127.0) * volume * 0.30;

                                // Re-trigger voice if note already ringing
                                let mut assigned = false;
                                for v in &mut voices {
                                    if v.active && v.note == note {
                                        v.freq = freq;
                                        v.target_gain = vel_gain;
                                        assigned = true;
                                        break;
                                    }
                                }

                                if !assigned {
                                    // Allocate free voice slot
                                    for v in &mut voices {
                                        if !v.active {
                                            v.note = note;
                                            v.freq = freq;
                                            v.target_gain = vel_gain;
                                            v.active = true;
                                            assigned = true;
                                            break;
                                        }
                                    }
                                }

                                if !assigned {
                                    // Voice stealing: reallocate the voice closest to silence
                                    let mut lowest_idx = 0;
                                    let mut lowest_gain = f32::MAX;
                                    for (idx, v) in voices.iter().enumerate() {
                                        if v.gain < lowest_gain {
                                            lowest_gain = v.gain;
                                            lowest_idx = idx;
                                        }
                                    }
                                    if let Some(v) = voices.get_mut(lowest_idx) {
                                        v.note = note;
                                        v.freq = freq;
                                        v.target_gain = vel_gain;
                                        v.active = true;
                                    }
                                }
                            }
                            SynthCommand::NoteOff { note } => {
                                for v in &mut voices {
                                    if v.active && v.note == note {
                                        v.target_gain = 0.0;
                                    }
                                }
                            }
                            SynthCommand::AllNotesOff => {
                                for v in &mut voices {
                                    v.target_gain = 0.0;
                                }
                            }
                            SynthCommand::Mute(new_muted) => {
                                muted = new_muted;
                            }
                            SynthCommand::Volume(new_vol) => {
                                volume = new_vol;
                            }
                        }
                    }

                    // 2. Synthesize audio frames directly with per-voice acoustic damper fall
                    let frames_needed = data.len().checked_div(channels).unwrap_or(0);

                    for i in 0..frames_needed {
                        let mut frame_sum = 0.0_f32;

                        for v in &mut voices {
                            if !v.active {
                                continue;
                            }

                            // Hexboard parity: snappy 5ms attack, 250ms acoustic release tail ("damper fall")
                            let coef = if v.target_gain > v.gain { 0.012_f32 } else { 0.0007_f32 };
                            v.gain += (v.target_gain - v.gain) * coef;

                            if v.gain < 0.0001 && v.target_gain == 0.0 {
                                v.active = false;
                                v.gain = 0.0;
                                continue;
                            }

                            frame_sum += (v.phase * 2.0 * std::f32::consts::PI).sin() * v.gain;
                            v.phase += v.freq / sample_rate;
                            if v.phase >= 1.0 {
                                v.phase -= 1.0;
                            }
                        }

                        let out_sample = if muted { 0.0 } else { frame_sum.clamp(-1.0, 1.0) };

                        if channels == 2 {
                            if let Some(s0) = data.get_mut(i * 2) {
                                *s0 = out_sample;
                            }
                            if let Some(s1) = data.get_mut(i * 2 + 1) {
                                *s1 = out_sample;
                            }
                        } else if channels == 1 {
                            if let Some(s0) = data.get_mut(i) {
                                *s0 = out_sample;
                            }
                        } else {
                            if let Some(s0) = data.get_mut(i * channels) {
                                *s0 = out_sample;
                            }
                            if let Some(s1) = data.get_mut(i * channels + 1) {
                                *s1 = out_sample;
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

    /// Triggers a note on event with velocity.
    pub fn note_on(&mut self, note: u8, velocity: u8) -> Result<(), SynthError> {
        self.tx
            .try_push(SynthCommand::NoteOn { note, velocity })
            .map_err(|_| SynthError::BufferFull)
    }

    /// Triggers a note off event for a specific note.
    pub fn note_off(&mut self, note: u8) -> Result<(), SynthError> {
        self.tx
            .try_push(SynthCommand::NoteOff { note })
            .map_err(|_| SynthError::BufferFull)
    }

    /// Releases all ringing notes with acoustic damper decay.
    pub fn all_notes_off(&mut self) -> Result<(), SynthError> {
        self.tx
            .try_push(SynthCommand::AllNotesOff)
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
        "Poly Sine"
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_calculation() {
        // A4 = MIDI 69 = 440.0 Hz
        let freq_a4 = 440.0 * 2.0_f32.powf((69.0 - 69.0) / 12.0);
        assert!((freq_a4 - 440.0).abs() < 1e-3);

        // A3 = MIDI 57 = 220.0 Hz
        let freq_a3 = 440.0 * 2.0_f32.powf((57.0 - 69.0) / 12.0);
        assert!((freq_a3 - 220.0).abs() < 1e-3);

        // C4 (Middle C) = MIDI 60 ≈ 261.6256 Hz
        let freq_c4 = 440.0 * 2.0_f32.powf((60.0 - 69.0) / 12.0);
        assert!((freq_c4 - 261.6256).abs() < 1e-2);
    }

    #[test]
    fn test_voice_allocation_and_release() {
        let mut voices = [Voice::default(); MAX_VOICES];
        let sample_rate = 44100.0_f32;

        // Trigger note 48 (C3)
        let note = 48;
        let freq = 440.0 * 2.0_f32.powf((f32::from(note) - 69.0) / 12.0);
        voices[0] = Voice {
            note,
            freq,
            phase: 0.0,
            gain: 0.0,
            target_gain: 0.3,
            active: true,
        };

        // Render 100 samples (attack phase)
        for _ in 0..100 {
            let coef = 0.012_f32;
            voices[0].gain += (voices[0].target_gain - voices[0].gain) * coef;
            voices[0].phase += voices[0].freq / sample_rate;
            if voices[0].phase >= 1.0 {
                voices[0].phase -= 1.0;
            }
        }
        assert!(voices[0].gain > 0.15);

        // Release note (damper fall)
        voices[0].target_gain = 0.0;
        for _ in 0..20000 {
            let coef = 0.0007_f32;
            voices[0].gain += (voices[0].target_gain - voices[0].gain) * coef;
            if voices[0].gain < 0.0001 {
                voices[0].active = false;
                break;
            }
        }
        assert!(!voices[0].active);
    }
}
