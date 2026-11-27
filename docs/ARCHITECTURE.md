# isokbd Architecture & Design

`isokbd` is designed according to the principles of safe, idiomatic, high-performance Rust (`rust-dev` standards).

---

## 1. System Topology

```text
 ┌─────────────────┐       ┌─────────────────┐       ┌────────────────────────┐
 │   Crossterm     │ ----> │    App State    │ ----> │  Ratatui TUI Renderer  │
 │  Event Stream   │       │ (Active Chords) │       │ (Grid + Harmony Panels)│
 └─────────────────┘       └────────┬────────┘       └────────────────────────┘
                                    │
                         ┌──────────┴──────────┐
                         │                     │
                         v                     v
                ┌─────────────────┐   ┌────────────────────────┐
                │   MIDI Engine   │   │  Synth Engine (Glicol) │
                │ (`midir` / ALSA)│   │ (CPAL + Lock-Free RB)  │
                └────────┬────────┘   └───────────┬────────────┘
                         │                        │
               ┌─────────┴─────────┐              v
               v                   v        [Audio Out / DAC]
       [RTRACK_MIDI_IN]    [Midi Through]  (Polyphonic Sine Init)
```

---

## 2. Module Breakdown

### `src/layout.rs`
- Pure geometric coordinate calculation.
- Defines physical QWERTY rows with stagger offsets.
- Defines the six `LayoutMode`s. Each mode is only an `(right, up_right)` interval pair plus a default root; one formula serves all of them.
- Implements `calculate_midi_note(row, col, root, mode)` with strict range clamping (`0..=127`).
- Implements `char_to_coord(c)` for fast character resolution across uppercase, lowercase, and symbols.

### `src/theory.rs`
- Translates raw MIDI note numbers to pitch class representations (`C`, `C#`, `D`...) and octave numbers.
- **Pitch-Class Kernel (`PitchClassSet`)**:
  - A `PitchClassSet(pub u16)` — a 12-bit vector where bit `0` is `C` and bit `11` is `B` — so "which degrees of the 12-tone circle am I holding" is one integer.
  - `from_midi_notes` folds any note list into the set, so octave doublings never inflate `count` (`k`).
  - `interval_formula` returns consecutive ascending deltas between the active degrees (C Maj7 -> `[4, 3, 4]`, the closing interval omitted by design).
  - `barcode_string` renders the 12 steps as `■` / `□`, and `complement` inverts the field (`!self.0 & 0x0FFF`) so `k + (12 - k) == 12` for every set.
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
- `#[cfg(test)] MidiEngine::silent()` is a connection-less engine whose sends are no-ops. Tests that only exercise application logic therefore run on a headless runner with no ALSA sequencer, instead of being `#[ignore]`d or quietly early-returning (which still reports as passed).

### `src/synth.rs`
- Built-in polyphonic synthesizer powered by the `glicol` graph engine and `cpal`.
- Decoupled from the TUI event loop via a lock-free single-producer single-consumer (SPSC) ring buffer (`ringbuf`).
- Implements a polyphonic "Sine Init" preset with $1/\sqrt{N}$ gain scaling across active notes to prevent digital clipping.
- Real-time mute toggling via `Enter` or `--no-synth` CLI flag.

### `src/app.rs`
- State manager holding `active_coords`, `active_notes`, `sustained_notes`, `theme`, and chord metadata.
- **Note lifecycle — event-driven, no decay timer**:
  - `KeyEventKind::Press` starts a note; `KeyEventKind::Release` stops it; typematic `Repeat` events are dropped outright so a held key never stutters.
  - `tick()` is deliberately a no-op. There is **no** inactivity timeout, so a note lives exactly as long as its `Release` does — and if the terminal never sends one (a multiplexer that drops the kitty release encoding), nothing rescues the note. The panic key (`Backspace` / `Delete`) and lifting the sustain pedal are the only ways out.
- **Themes**: `Theme` (`Catppuccin` default, `GeneseLufs`, `GeneseDr`, `ClassicAnsi`) resolves to a `ColorPalette` of semantic tokens (`bg`, `lead`, `accent`, `root`, `natural_fg`, `accidental_fg`, `held_bg`, `held_fg`, `sustain_bg`). Nothing downstream names a raw colour.

### `src/ui.rs`
- Layout bands: header (3 rows), keyboard grid (flexible), harmonic panels (height scales with the terminal), footer (3 rows).
- **Pad grid**: one `PadCell` per key carrying its own illumination state, so the renderer can pick a geometry — box pads at `>= 92×14`, then three compact densities — without the caller knowing.
- **Harmony panels**: `render_chord_panel` (chord name, interval formula, `k/12` density, 12-step barcode, active notes) and `render_dodecagram_panel` (12-point ring with the chord polygon and the modal complement, collapsing to a compact clock below 100 columns).
- Every highlight is resolved from the active `ColorPalette`; the renderer hardcodes no ANSI colour.
- The render path allocates per cell (`format!` + owned `String`) and builds a `Vec` per row. That is the honest cost of the `PadCell<'a>` / themed-span shape and the reason the spec's "non-allocating hot loop" criterion is recorded as **not met** (`docs/specs/2026-10-31T0154Z_.../SPEC.md` §7). No frame-rate regression has been measured at 16 ms ticks.

### `src/main.rs`
- Terminal lifecycle coordinator:
  - Pushes the Kitty Keyboard Protocol progressive enhancement flags (`DISAMBIGUATE_ESCAPE_CODES`, `REPORT_ALL_KEYS_AS_ESCAPE_CODES`, `REPORT_EVENT_TYPES`) — this is what makes `KeyRelease` exist at all.
  - Installs a panic hook to guarantee terminal raw mode is always cleanly restored upon unexpected abort.
  - ~60 FPS loop: `event::poll` blocks for up to a 16 ms tick, then redraws. No busy-wait.

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
