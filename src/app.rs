//! Main application state, event handling, and inactivity decay loop.

use std::collections::{BTreeSet, HashSet};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::layout::{calculate_midi_note, char_to_coord, LayoutMode};
use crate::midi::MidiEngine;
use crate::synth::SynthEngine;
use crate::theory::{detect_chord, note_to_name};

pub struct App {
    pub layout_mode: LayoutMode,
    pub root_note: u8, // Base MIDI note for (row: 0, col: 0)
    pub velocity: u8,
    pub is_space_held: bool, // True while Space sustain pedal is physically depressed
    pub active_coords: HashSet<(usize, usize)>,
    pub active_notes: BTreeSet<u8>,
    pub sustained_notes: HashSet<u8>,
    pub midi: MidiEngine,
    pub synth: Option<SynthEngine>,
    pub detected_chord: Option<String>,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(midi: MidiEngine, synth: Option<SynthEngine>) -> Self {
        let initial_layout = LayoutMode::WickiHayden;
        Self {
            layout_mode: initial_layout,
            root_note: initial_layout.default_base(), // 41 (F2), placing C3 on Home-row 'S'
            velocity: 100,
            is_space_held: false,
            active_coords: HashSet::new(),
            active_notes: BTreeSet::new(),
            sustained_notes: HashSet::new(),
            midi,
            synth,
            detected_chord: None,
            status_message: None,
            should_quit: false,
        }
    }

    /// Periodic frame tick.
    pub fn tick(&mut self) {
        // No-op: true note lifecycles are driven by physical KeyRelease events
        // rather than arbitrary timeout decay hacks.
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        // Intercept global quit shortcuts
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        // Hexboard parity: ignore OS typematic auto-repeat completely so held notes never stutter!
        if event.kind == KeyEventKind::Repeat {
            return;
        }

        match event.kind {
            KeyEventKind::Press => self.handle_key_press(event),
            KeyEventKind::Release => self.handle_key_release(event),
            KeyEventKind::Repeat => {}
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
                self.root_note = self.layout_mode.default_base();
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
                self.sync_synth_volume();
                self.status_message = Some(format!("Velocity: {}", self.velocity));
            }
            KeyCode::PageDown => {
                self.velocity = self.velocity.saturating_sub(10).max(10);
                self.sync_synth_volume();
                self.status_message = Some(format!("Velocity: {}", self.velocity));
            }
            KeyCode::Char(' ') => {
                self.is_space_held = true;
                self.status_message = Some("Sustain Pedal: ENGAGED (Hold Space & tap notes)".to_string());
            }
            KeyCode::Enter => {
                if let Some(ref mut s) = self.synth {
                    match s.toggle_mute() {
                        Ok(muted) => {
                            self.status_message = Some(format!(
                                "Internal Synth: {}",
                                if muted { "MUTED" } else { "ACTIVE (Sine Init)" }
                            ));
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Synth error: {e}"));
                        }
                    }
                } else {
                    self.status_message = Some("Internal synth not active (MIDI-only mode)".to_string());
                }
            }
            KeyCode::Char(c) => {
                if let Some((row, col)) = char_to_coord(c)
                    && let Some(note) = calculate_midi_note(row, col, self.root_note, self.layout_mode)
                {
                    let is_new_coord = self.active_coords.insert((row, col));
                    if is_new_coord && !self.active_notes.contains(&note) {
                        let _ = self.midi.send_note_on(note, self.velocity);
                        if let Some(ref mut s) = self.synth {
                            let _ = s.note_on(note, self.velocity);
                        }
                        self.active_notes.insert(note);
                        self.update_chord_detection();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_key_release(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(' ') => {
                self.is_space_held = false;
                self.release_sustained_notes();
                self.status_message = Some("Sustain Pedal: RELEASED".to_string());
            }
            KeyCode::Char(c) => {
                if let Some((row, col)) = char_to_coord(c) {
                    self.release_coord(row, col);
                }
            }
            _ => {}
        }
    }

    fn release_coord(&mut self, row: usize, col: usize) {
        self.active_coords.remove(&(row, col));
        if let Some(note) = calculate_midi_note(row, col, self.root_note, self.layout_mode) {
            if self.is_space_held {
                self.sustained_notes.insert(note);
            } else {
                let _ = self.midi.send_note_off(note);
                if let Some(ref mut s) = self.synth {
                    let _ = s.note_off(note);
                }
                self.active_notes.remove(&note);
                self.update_chord_detection();
            }
        }
    }

    fn release_sustained_notes(&mut self) {
        let currently_held_notes: HashSet<u8> = self
            .active_coords
            .iter()
            .filter_map(|&(r, c)| calculate_midi_note(r, c, self.root_note, self.layout_mode))
            .collect();

        for note in self.sustained_notes.drain() {
            if !currently_held_notes.contains(&note) {
                let _ = self.midi.send_note_off(note);
                if let Some(ref mut s) = self.synth {
                    let _ = s.note_off(note);
                }
                self.active_notes.remove(&note);
            }
        }
        self.update_chord_detection();
    }

    pub fn all_notes_off(&mut self) {
        let _ = self.midi.all_notes_off();
        if let Some(ref mut s) = self.synth {
            let _ = s.all_notes_off();
        }
        self.active_coords.clear();
        self.active_notes.clear();
        self.sustained_notes.clear();
        self.detected_chord = None;
    }

    fn sync_synth_volume(&mut self) {
        if let Some(ref mut s) = self.synth {
            let scaled_vol = (f32::from(self.velocity) / 127.0) * 0.45;
            let _ = s.set_volume(scaled_vol);
        }
    }

    fn update_chord_detection(&mut self) {
        let notes_vec: Vec<u8> = self.active_notes.iter().copied().collect();
        self.detected_chord = detect_chord(&notes_vec);
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_momentary_sustain_pedal_stacking() {
        let midi = MidiEngine::new().expect("Failed to create MIDI engine for test");
        let mut app = App::new(midi, None);

        // 1. Depress Spacebar (sustain pedal engaged)
        app.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(app.is_space_held);

        // 2. Tap 's' (Row 1, Col 1: C3 = 48 in WickiHayden with base 41)
        let key_s = KeyCode::Char('s');
        app.handle_key_event(KeyEvent::new(key_s, KeyModifiers::NONE)); // press
        assert!(app.active_notes.contains(&48));

        app.handle_key_event(KeyEvent {
            code: key_s,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        });
        // Key is physically released, BUT pedal is held, so note stays sounding!
        assert!(app.active_notes.contains(&48));
        assert!(app.sustained_notes.contains(&48));

        // 3. Tap 'f' (Row 1, Col 3: E3 = 52 in WickiHayden)
        let key_f = KeyCode::Char('f');
        app.handle_key_event(KeyEvent::new(key_f, KeyModifiers::NONE)); // press
        app.handle_key_event(KeyEvent {
            code: key_f,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        });
        // Both notes stacked and ringing!
        assert!(app.active_notes.contains(&48));
        assert!(app.active_notes.contains(&52));
        assert_eq!(app.active_notes.len(), 2);

        // 4. Release Spacebar (pedal released)
        app.handle_key_event(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        });
        assert!(!app.is_space_held);
        // All sustained notes released!
        assert!(app.active_notes.is_empty());
        assert!(app.sustained_notes.is_empty());
    }

    #[test]
    fn test_pedal_release_preserves_physically_held_keys() {
        let midi = MidiEngine::new().expect("Failed to create MIDI engine for test");
        let mut app = App::new(midi, None);

        // Depress Spacebar
        app.handle_key_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

        // Physically hold 's' (C3 = 48)
        let key_s = KeyCode::Char('s');
        app.handle_key_event(KeyEvent::new(key_s, KeyModifiers::NONE));

        // Tap and release 'f' (E3 = 52)
        let key_f = KeyCode::Char('f');
        app.handle_key_event(KeyEvent::new(key_f, KeyModifiers::NONE));
        app.handle_key_event(KeyEvent {
            code: key_f,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        });

        // Release pedal while 's' is STILL physically held down
        app.handle_key_event(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        });

        // 'f' was released so it dies, but 's' is still held so it MUST keep sounding!
        assert!(!app.active_notes.contains(&52));
        assert!(app.active_notes.contains(&48));

        // Now release 's'
        app.handle_key_event(KeyEvent {
            code: key_s,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        });
        assert!(app.active_notes.is_empty());
    }
}
