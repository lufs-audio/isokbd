# isokbd Architecture & Design

`isokbd` is designed according to the principles of safe, idiomatic, high-performance Rust (`rust-dev` standards).

---

## 1. System Topology

```text
 ┌─────────────────┐       ┌─────────────────┐       ┌────────────────────────┐
 │   Crossterm     │ ----> │    App State    │ ----> │  Ratatui TUI Renderer  │
 │  Event Stream   │       │ (Active Chords) │       │ (Illuminated 2D Grid)  │
 └─────────────────┘       └─────────────────┘       └────────────────────────┘
                                    │
                                    v
                           ┌─────────────────┐
                           │   MIDI Engine   │
                           │ (`midir` / ALSA)│
                           └─────────────────┘
                                    │
                         ┌──────────┴──────────┐
                         v                     v
                 [RTRACK_MIDI_IN]    [Midi Through Port-0]
```

---

## 2. Module Breakdown

### `src/layout.rs`
- Pure geometric coordinate calculation.
- Defines physical QWERTY rows with stagger offsets.
- Implements `calculate_midi_note(row, col, root, mode)` with strict range clamping (`0..=127`).
- Implements `char_to_coord(c)` for fast character resolution across uppercase, lowercase, and symbols.

### `src/theory.rs`
- Translates raw MIDI note numbers to pitch class representations (`C`, `C#`, `D`...) and octave numbers.
- **Chord Recognition Engine**:
  - Extracts unique pitch classes modulo 12.
  - Rotates through candidate roots and tests interval vectors against chord dictionaries:
    - Dyads: 5ths, 4ths, 3rds
    - Triads: Major, Minor, Diminished, Augmented, Sus2, Sus4
    - 7ths: Maj7, Min7, Dominant 7th, Half-Diminished, Diminished 7th, mMaj7
    - Extensions: 6ths, Add9

### `src/midi.rs`
- Multi-port real-time MIDI dispatcher using `midir`.
- Automatically executes ALSA bridging (`aconnect`) to link `Midi Through` to `RTRACK_MIDI_IN`.
- Broadcasts Note-On, Note-Off, and All-Notes-Off (CC 123) across all open port connections.

### `src/app.rs`
- State manager holding `active_coords`, `active_notes`, `sustained_notes`, and chord metadata.
- **Decay Engine (`tick`)**:
  - Implements the 320ms typematic repeat timeout window to ensure smooth polyphonic sustain in terminal environments lacking release events.
  - Automatically clears expired coordinates.

### `src/ui.rs`
- Staggered ASCII/terminal grid rendering.
- Dynamically highlights active keys with inverted bold styling.
- Renders pitch-class harmonic cues:
  - Cyan: Root notes
  - White: Natural notes
  - Dark Gray: Accidentals (sharps / flats)

### `src/main.rs`
- Terminal lifecycle coordinator:
  - Pushes Kitty Keyboard Protocol progressive enhancement flags.
  - Installs a panic hook to guarantee terminal raw mode is always cleanly restored upon unexpected abort.
  - 60 FPS event loop with zero blocking.

---

## 3. Strict Compiler & Zero-Panic Enforcement

The crate enforces zero unwraps in application code via `Cargo.toml`:
```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
indexing_slicing = "warn"
```

All potential failure points propagate via `thiserror` (library domain) and `anyhow` (application root).
