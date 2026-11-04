//! MIDI Output engine leveraging `midir`.

use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MidiError {
    #[error("Failed to initialize MIDI output: {0}")]
    Init(String),
    #[error("Failed to connect to MIDI port: {0}")]
    Connect(String),
    #[error("Failed to send MIDI message: {0}")]
    Send(String),
}

pub struct MidiEngine {
    connections: Vec<MidiOutputConnection>,
    port_names: Vec<String>,
    channel: u8, // 0-indexed (0..15)
}

#[cfg(test)]
impl MidiEngine {
    /// A connection-less engine for tests: sends are no-ops, so application
    /// logic that does not depend on a live MIDI backend can be verified
    /// anywhere, including headless CI runners with no ALSA sequencer.
    #[must_use]
    pub(crate) fn silent() -> Self {
        Self {
            connections: Vec::new(),
            port_names: Vec::new(),
            channel: 0,
        }
    }
}

impl MidiEngine {
    /// Initializes MIDI connection. Connects to all available destination ports
    /// (e.g. `RTRACK_MIDI_IN`, `Midi Through Port-0`), and bridges ALSA ports if rtrack is active.
    pub fn new() -> Result<Self, MidiError> {
        // Attempt ALSA bridge to rtrack if both exist
        let _ = Command::new("aconnect")
            .args(["Midi Through", "RTRACK_MIDI_IN"])
            .output();

        let midi_out =
            MidiOutput::new("isokbd-probe").map_err(|e| MidiError::Init(e.to_string()))?;

        let ports = midi_out.ports();
        let mut connections = Vec::new();
        let mut port_names = Vec::new();

        for (idx, port) in ports.iter().enumerate() {
            let name = midi_out
                .port_name(port)
                .unwrap_or_else(|_| format!("Port {idx}"));

            if name.contains("isokbd") {
                continue;
            }

            if let Ok(out) = MidiOutput::new(&format!("isokbd-out-{idx}"))
                && let Ok(conn) = out.connect(port, &format!("isokbd-client-{idx}"))
            {
                connections.push(conn);
                port_names.push(name);
            }
        }

        if connections.is_empty() {
            // Virtual port fallback on Linux
            let out = MidiOutput::new("isokbd").map_err(|e| MidiError::Init(e.to_string()))?;
            let conn = out
                .create_virtual("isokbd")
                .map_err(|e| MidiError::Connect(e.to_string()))?;
            connections.push(conn);
            port_names.push("isokbd (virtual)".to_string());
        }

        Ok(Self {
            connections,
            port_names,
            channel: 0,
        })
    }

    #[must_use]
    pub fn port_name(&self) -> String {
        if self.port_names.is_empty() {
            "None".to_string()
        } else {
            self.port_names.join(", ")
        }
    }

    pub fn send_note_on(&mut self, note: u8, velocity: u8) -> Result<(), MidiError> {
        let status = 0x90 | (self.channel & 0x0F);
        let msg = [status, note & 0x7F, velocity & 0x7F];
        for conn in &mut self.connections {
            conn.send(&msg)
                .map_err(|e| MidiError::Send(e.to_string()))?;
        }
        Ok(())
    }

    pub fn send_note_off(&mut self, note: u8) -> Result<(), MidiError> {
        let status = 0x80 | (self.channel & 0x0F);
        let msg = [status, note & 0x7F, 0];
        for conn in &mut self.connections {
            conn.send(&msg)
                .map_err(|e| MidiError::Send(e.to_string()))?;
        }
        Ok(())
    }

    pub fn all_notes_off(&mut self) -> Result<(), MidiError> {
        // Control Change 123 (All Notes Off)
        let status = 0xB0 | (self.channel & 0x0F);
        let msg = [status, 123, 0];
        for conn in &mut self.connections {
            conn.send(&msg)
                .map_err(|e| MidiError::Send(e.to_string()))?;
        }
        Ok(())
    }
}
