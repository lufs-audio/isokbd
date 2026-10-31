# Unit 02 — pitch-class-vector-and-interval-barcode

## Objective
Implement the 12-bit pitch-class set (`PitchClassSet`) in `src/theory.rs` and render the Genèse 12-step interval barcode widget in the active chord panel.

## Context
- `src/theory.rs` currently detects basic chord triads via `detect_chord()` returning `Option<String>`.
- In `genese-dodecaphonique-assets`, all pitch sets are derived from 12-bit vectors, interval cells, and semitone formulas.
- An interval barcode (`■ □ □ □ ■ □ □ ■ □ □ □ ■`) provides instantaneous visual feedback on chord structure and interval spacing.
- See phase spec: `docs/specs/2026-10-31T0154Z_genese-tactile-grid-and-harmonic-matrix-ui/SPEC.md`.

## Acceptance criteria
- [ ] Define `PitchClassSet(pub u16)` where bits `0..11` represent pitch classes relative to C (`0 = C`, `1 = C#`, ..., `11 = B`).
- [ ] Implement `PitchClassSet::from_midi_notes(notes: &[u8])`.
- [ ] Implement `PitchClassSet::count(&self) -> u8` returning degree count `k` (0..12).
- [ ] Implement `PitchClassSet::interval_formula(&self) -> Vec<u8>` returning semitone distances between active pitch classes.
- [ ] Implement `PitchClassSet::barcode_string(&self) -> String` producing 12-glyph string (e.g. `■ □ □ □ ■ □ □ ■ □ □ □ ■`).
- [ ] In `src/ui.rs`, update `render_chord_panel()` to display:
  - The interval barcode.
  - The semitone formula (e.g. `[4, 3, 5]`).
  - Degree density ratio (`k / 12`).
- [ ] Comprehensive unit tests verify chord conversions, interval calculations, and bitmask operations.

## Interface contract
```rust
// In src/theory.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PitchClassSet(pub u16);

impl PitchClassSet {
    #[must_use]
    pub fn from_midi_notes(notes: &[u8]) -> Self;
    #[must_use]
    pub fn count(&self) -> u8;
    #[must_use]
    pub fn contains(&self, pc: u8) -> bool;
    #[must_use]
    pub fn interval_formula(&self) -> Vec<u8>;
    #[must_use]
    pub fn barcode_string(&self) -> String;
}
```

## Boundaries
- **Touches:**
  - `src/theory.rs`
  - `src/ui.rs`
  - `tests/`
- **Does NOT touch:**
  - `src/synth.rs`
  - `src/midi.rs`
  - `src/layout.rs`

## Verification
```bash
cargo test theory::tests
```
