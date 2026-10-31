//! Main application state, event handling, and inactivity decay loop.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::style::Color;
use std::collections::{BTreeSet, HashSet};

use crate::layout::{LayoutMode, calculate_midi_note, char_to_coord};
use crate::midi::MidiEngine;
use crate::synth::SynthEngine;
use crate::theory::{detect_chord, note_to_name};

/// Runtime-selectable color themes for the terminal instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// LUFS ecosystem palette: charcoal ground, teal lead, gold accent.
    #[default]
    GeneseLufs,
    /// DR ecosystem palette: warm near-black ground, gold lead, plum accent.
    GeneseDr,
    /// The original basic ANSI colors.
    ClassicAnsi,
}

/// Semantic color tokens resolved from the active [`Theme`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPalette {
    /// Base background / ground.
    pub bg: Color,
    /// Primary active color (teal or gold).
    pub lead: Color,
    /// Secondary active color (gold or plum).
    pub accent: Color,
    /// Root-key indicator.
    pub root: Color,
    /// Natural key surface text.
    pub natural_fg: Color,
    /// Accidental (sharp/flat) key surface text.
    pub accidental_fg: Color,
    /// Background of a physically held key.
    pub held_bg: Color,
    /// Foreground of a physically held key.
    pub held_fg: Color,
    /// Background of a key ringing under the sustain pedal.
    pub sustain_bg: Color,
}

impl Theme {
    /// The next theme in the cycle.
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::GeneseLufs => Self::GeneseDr,
            Self::GeneseDr => Self::ClassicAnsi,
            Self::ClassicAnsi => Self::GeneseLufs,
        }
    }

    /// Human-readable theme name shown in the header bar.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::GeneseLufs => "Genese LUFS",
            Self::GeneseDr => "Genese DR",
            Self::ClassicAnsi => "Classic ANSI",
        }
    }

    /// Resolves the theme's semantic color tokens.
    #[must_use]
    pub fn palette(&self) -> ColorPalette {
        match self {
            Self::GeneseLufs => ColorPalette {
                bg: Color::Rgb(0x11, 0x11, 0x11),
                lead: Color::Rgb(0x78, 0xBE, 0xBA),
                accent: Color::Rgb(0xD9, 0xA2, 0x3D),
                root: Color::Rgb(0xD9, 0xA2, 0x3D),
                natural_fg: Color::Rgb(0xE6, 0xE6, 0xE6),
                accidental_fg: Color::Rgb(0x6A, 0x6A, 0x6A),
                held_bg: Color::Rgb(0x78, 0xBE, 0xBA),
                held_fg: Color::Rgb(0x11, 0x11, 0x11),
                sustain_bg: Color::Rgb(0xC4, 0x45, 0x7E),
            },
            Self::GeneseDr => ColorPalette {
                bg: Color::Rgb(0x1A, 0x1A, 0x18),
                lead: Color::Rgb(0xD9, 0xA2, 0x3D),
                accent: Color::Rgb(0x9E, 0x73, 0x86),
                root: Color::Rgb(0xD9, 0xA2, 0x3D),
                natural_fg: Color::Rgb(0xD8, 0xD2, 0xC4),
                accidental_fg: Color::Rgb(0x6E, 0x64, 0x59),
                held_bg: Color::Rgb(0xD9, 0xA2, 0x3D),
                held_fg: Color::Rgb(0x1A, 0x1A, 0x18),
                sustain_bg: Color::Rgb(0x9E, 0x73, 0x86),
            },
            Self::ClassicAnsi => ColorPalette {
                bg: Color::Black,
                lead: Color::Cyan,
                accent: Color::Yellow,
                root: Color::Yellow,
                natural_fg: Color::White,
                accidental_fg: Color::DarkGray,
                held_bg: Color::Yellow,
                held_fg: Color::Black,
                sustain_bg: Color::Green,
            },
        }
    }
}

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
    pub theme: Theme,
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
            theme: Theme::default(),
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
                self.status_message =
                    Some(format!("Switched layout to {}", self.layout_mode.name()));
            }
            KeyCode::Up => {
                if self.root_note <= 115 {
                    self.root_note += 12;
                    self.all_notes_off();
                    self.status_message = Some(format!(
                        "Octave Up (Root: {})",
                        note_to_name(self.root_note)
                    ));
                }
            }
            KeyCode::Down => {
                if self.root_note >= 12 {
                    self.root_note -= 12;
                    self.all_notes_off();
                    self.status_message = Some(format!(
                        "Octave Down (Root: {})",
                        note_to_name(self.root_note)
                    ));
                }
            }
            KeyCode::Right => {
                if self.root_note < 127 {
                    self.root_note += 1;
                    self.all_notes_off();
                    self.status_message = Some(format!(
                        "Transpose Up (Root: {})",
                        note_to_name(self.root_note)
                    ));
                }
            }
            KeyCode::Left => {
                if self.root_note > 0 {
                    self.root_note -= 1;
                    self.all_notes_off();
                    self.status_message = Some(format!(
                        "Transpose Down (Root: {})",
                        note_to_name(self.root_note)
                    ));
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
                self.status_message =
                    Some("Sustain Pedal: ENGAGED (Hold Space & tap notes)".to_string());
            }
            KeyCode::F(2) | KeyCode::Char('T') => {
                self.theme = self.theme.next();
                self.status_message = Some(format!("Theme: {}", self.theme.name()));
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
                    self.status_message =
                        Some("Internal synth not active (MIDI-only mode)".to_string());
                }
            }
            KeyCode::Char(c) => {
                if let Some((row, col)) = char_to_coord(c)
                    && let Some(note) =
                        calculate_midi_note(row, col, self.root_note, self.layout_mode)
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
    fn theme_cycles_through_all_variants() {
        assert_eq!(Theme::default(), Theme::GeneseLufs);
        assert_eq!(Theme::GeneseLufs.next(), Theme::GeneseDr);
        assert_eq!(Theme::GeneseDr.next(), Theme::ClassicAnsi);
        assert_eq!(Theme::ClassicAnsi.next(), Theme::GeneseLufs);
        // A full cycle returns to the start.
        assert_eq!(Theme::default().next().next().next(), Theme::default());
        assert_eq!(Theme::GeneseLufs.name(), "Genese LUFS");
        assert_eq!(Theme::GeneseDr.name(), "Genese DR");
        assert_eq!(Theme::ClassicAnsi.name(), "Classic ANSI");
    }

    #[test]
    fn theme_palettes_use_documented_tokens() {
        let lufs = Theme::GeneseLufs.palette();
        assert_eq!(lufs.bg, Color::Rgb(0x11, 0x11, 0x11));
        assert_eq!(lufs.lead, Color::Rgb(0x78, 0xBE, 0xBA));
        assert_eq!(lufs.accent, Color::Rgb(0xD9, 0xA2, 0x3D));
        assert_eq!(lufs.sustain_bg, Color::Rgb(0xC4, 0x45, 0x7E));

        let dr = Theme::GeneseDr.palette();
        assert_eq!(dr.bg, Color::Rgb(0x1A, 0x1A, 0x18));
        assert_eq!(dr.lead, Color::Rgb(0xD9, 0xA2, 0x3D));
        assert_eq!(dr.accent, Color::Rgb(0x9E, 0x73, 0x86));

        // The ANSI fallback keeps the original terminal colors.
        let ansi = Theme::ClassicAnsi.palette();
        assert_eq!(ansi.lead, Color::Cyan);
        assert_eq!(ansi.accent, Color::Yellow);
        assert_eq!(ansi.natural_fg, Color::White);
    }

    #[test]
    fn theme_shortcut_cycles_and_reports() {
        let mut app = App::new(MidiEngine::silent(), None);
        assert_eq!(app.theme, Theme::GeneseLufs);
        app.handle_key_event(KeyEvent::new(KeyCode::Char('T'), KeyModifiers::SHIFT));
        assert_eq!(app.theme, Theme::GeneseDr);
        assert_eq!(app.status_message.as_deref(), Some("Theme: Genese DR"));
        app.handle_key_event(KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));
        assert_eq!(app.theme, Theme::ClassicAnsi);
    }

    #[test]
    fn test_momentary_sustain_pedal_stacking() {
        let midi = MidiEngine::silent();
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
        let midi = MidiEngine::silent();
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
