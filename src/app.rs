//! Main application state, event handling, and inactivity decay loop.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::{Duration, Instant};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::layout::{calculate_midi_note, char_to_coord, LayoutMode};
use crate::midi::MidiEngine;
use crate::theory::{detect_chord, note_to_name};

pub struct App {
    pub layout_mode: LayoutMode,
    pub root_note: u8, // Base MIDI note for (row: 0, col: 0)
    pub velocity: u8,
    pub sustain_enabled: bool,
    pub active_coords: HashMap<(usize, usize), Instant>,
    pub active_notes: BTreeSet<u8>,
    pub sustained_notes: HashSet<u8>,
    pub midi: MidiEngine,
    pub detected_chord: Option<String>,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(midi: MidiEngine) -> Self {
        Self {
            layout_mode: LayoutMode::AllFourths,
            root_note: 48, // C3
            velocity: 100,
            sustain_enabled: false,
            active_coords: HashMap::new(),
            active_notes: BTreeSet::new(),
            sustained_notes: HashSet::new(),
            midi,
            detected_chord: None,
            status_message: None,
            should_quit: false,
        }
    }

    /// Periodic tick called every frame (~16ms).
    /// Detects key releases in terminals or multiplexers (like Herdr/tmux) that do not
    /// forward the Kitty keyboard protocol KeyRelease escape sequence.
    pub fn tick(&mut self) {
        let now = Instant::now();
        // A typematic key repeat on Linux usually fires every 30-35ms.
        // If 150ms elapse with no new event, the user has let go of the physical key.
        let decay_timeout = Duration::from_millis(150);

        let expired: Vec<(usize, usize)> = self
            .active_coords
            .iter()
            .filter(|(_, last_seen)| now.duration_since(**last_seen) > decay_timeout)
            .map(|(&coord, _)| coord)
            .collect();

        for (row, col) in expired {
            self.release_coord(row, col);
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        // Intercept global quit shortcuts
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match event.kind {
            KeyEventKind::Press => self.handle_key_press(event),
            KeyEventKind::Release => self.handle_key_release(event),
            KeyEventKind::Repeat => self.handle_key_repeat(event),
        }
    }

    fn handle_key_press(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Backspace | KeyCode::Delete => {
                self.all_notes_off();
                self.status_message = Some("All notes silenced".to_string());
            }
            KeyCode::Tab => {
                self.layout_mode = self.layout_mode.next();
                self.all_notes_off();
                self.status_message = Some(format!("Switched layout to {}", self.layout_mode.name()));
            }
            KeyCode::Up => {
                if self.root_note <= 115 {
                    self.root_note += 12;
                    self.all_notes_off();
                    self.status_message = Some(format!("Octave Up (Root: {})", note_to_name(self.root_note)));
                }
            }
            KeyCode::Down => {
                if self.root_note >= 12 {
                    self.root_note -= 12;
                    self.all_notes_off();
                    self.status_message = Some(format!("Octave Down (Root: {})", note_to_name(self.root_note)));
                }
            }
            KeyCode::Right => {
                if self.root_note < 127 {
                    self.root_note += 1;
                    self.all_notes_off();
                    self.status_message = Some(format!("Transpose Up (Root: {})", note_to_name(self.root_note)));
                }
            }
            KeyCode::Left => {
                if self.root_note > 0 {
                    self.root_note -= 1;
                    self.all_notes_off();
                    self.status_message = Some(format!("Transpose Down (Root: {})", note_to_name(self.root_note)));
                }
            }
            KeyCode::PageUp => {
                self.velocity = (self.velocity + 10).min(127);
                self.status_message = Some(format!("Velocity: {}", self.velocity));
            }
            KeyCode::PageDown => {
                self.velocity = self.velocity.saturating_sub(10).max(10);
                self.status_message = Some(format!("Velocity: {}", self.velocity));
            }
            KeyCode::Char(' ') => {
                self.sustain_enabled = !self.sustain_enabled;
                if !self.sustain_enabled {
                    self.release_sustained_notes();
                }
                self.status_message = Some(format!(
                    "Sustain: {}",
                    if self.sustain_enabled { "ON" } else { "OFF" }
                ));
            }
            KeyCode::Char(c) => {
                if let Some((row, col)) = char_to_coord(c)
                    && let Some(note) = calculate_midi_note(row, col, self.root_note, self.layout_mode)
                {
                    self.active_coords.insert((row, col), Instant::now());
                    if !self.active_notes.contains(&note) {
                        let _ = self.midi.send_note_on(note, self.velocity);
                        self.active_notes.insert(note);
                        self.update_chord_detection();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_key_repeat(&mut self, event: KeyEvent) {
        if let KeyCode::Char(c) = event.code
            && let Some((row, col)) = char_to_coord(c)
        {
            // Update timestamp to keep the note active while key is held down
            self.active_coords.insert((row, col), Instant::now());
        }
    }

    fn handle_key_release(&mut self, event: KeyEvent) {
        if let KeyCode::Char(c) = event.code
            && let Some((row, col)) = char_to_coord(c)
        {
            self.release_coord(row, col);
        }
    }

    fn release_coord(&mut self, row: usize, col: usize) {
        self.active_coords.remove(&(row, col));
        if let Some(note) = calculate_midi_note(row, col, self.root_note, self.layout_mode) {
            if self.sustain_enabled {
                self.sustained_notes.insert(note);
            } else {
                let _ = self.midi.send_note_off(note);
                self.active_notes.remove(&note);
                self.update_chord_detection();
            }
        }
    }

    fn release_sustained_notes(&mut self) {
        let currently_held_notes: HashSet<u8> = self
            .active_coords
            .keys()
            .filter_map(|&(r, c)| calculate_midi_note(r, c, self.root_note, self.layout_mode))
            .collect();

        for note in self.sustained_notes.drain() {
            if !currently_held_notes.contains(&note) {
                let _ = self.midi.send_note_off(note);
                self.active_notes.remove(&note);
            }
        }
        self.update_chord_detection();
    }

    pub fn all_notes_off(&mut self) {
        let _ = self.midi.all_notes_off();
        self.active_coords.clear();
        self.active_notes.clear();
        self.sustained_notes.clear();
        self.detected_chord = None;
    }

    fn update_chord_detection(&mut self) {
        let notes_vec: Vec<u8> = self.active_notes.iter().copied().collect();
        self.detected_chord = detect_chord(&notes_vec);
    }
}
