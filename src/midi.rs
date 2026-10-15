//! MIDI Output engine leveraging `midir`.

use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
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
    connection: MidiOutputConnection,
    port_name: String,
    channel: u8, // 0-indexed (0..15)
}

impl MidiEngine {
    /// Initializes MIDI connection. Prefers "Midi Through" port on Linux if available,
    /// otherwise falls back to creating a virtual ALSA port named "isokbd".
    pub fn new() -> Result<Self, MidiError> {
        let midi_out = MidiOutput::new("isokbd")
            .map_err(|e| MidiError::Init(e.to_string()))?;

        let ports = midi_out.ports();

        // Check if there is an existing "Midi Through" port
        let through_port = ports.iter().find(|p| {
            midi_out
                .port_name(p)
                .map(|name| name.contains("Midi Through"))
                .unwrap_or(false)
        });

        if let Some(port) = through_port {
            let name = midi_out
                .port_name(port)
                .unwrap_or_else(|_| "Midi Through".to_string());
            let connection = midi_out
                .connect(port, "isokbd-client")
                .map_err(|e| MidiError::Connect(e.to_string()))?;

            Ok(Self {
                connection,
                port_name: name,
                channel: 0,
            })
        } else if let Some(first_port) = ports.first() {
            let name = midi_out
                .port_name(first_port)
                .unwrap_or_else(|_| "Default Port".to_string());
            let connection = midi_out
                .connect(first_port, "isokbd-client")
                .map_err(|e| MidiError::Connect(e.to_string()))?;

            Ok(Self {
                connection,
                port_name: name,
                channel: 0,
            })
        } else {
            // Virtual port fallback on Linux
            let connection = midi_out
                .create_virtual("isokbd")
                .map_err(|e| MidiError::Connect(e.to_string()))?;

            Ok(Self {
                connection,
                port_name: "Virtual Port (isokbd)".to_string(),
                channel: 0,
            })
        }
    }

    #[must_use]
    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    pub fn send_note_on(&mut self, note: u8, velocity: u8) -> Result<(), MidiError> {
        let status = 0x90 | (self.channel & 0x0F);
        self.connection
            .send(&[status, note & 0x7F, velocity & 0x7F])
            .map_err(|e| MidiError::Send(e.to_string()))
    }

    pub fn send_note_off(&mut self, note: u8) -> Result<(), MidiError> {
        let status = 0x80 | (self.channel & 0x0F);
        self.connection
            .send(&[status, note & 0x7F, 0])
            .map_err(|e| MidiError::Send(e.to_string()))
    }

    pub fn all_notes_off(&mut self) -> Result<(), MidiError> {
        // Control Change 123 (All Notes Off)
        let status = 0xB0 | (self.channel & 0x0F);
        self.connection
            .send(&[status, 123, 0])
            .map_err(|e| MidiError::Send(e.to_string()))
    }
}
