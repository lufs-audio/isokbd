# Unit 03 — complementarity-dodecagram-widget

## Objective
Implement a dedicated visual harmonic analysis widget rendering a 12-point circular pitch-class dodecagram and identifying the Genèse modal complement (`12 - k`).

## Context
- In `genese-dodecaphonique-assets`, the dodecagram renders chords and scales as geometric polygons on twelve positions, and the complementarity ring visualizes the relationship between a modal set and its antipode.
- On an isomorphic keyboard, geometric symmetries (major triad triangle, diminished seventh square, whole tone hexagon) are directly playable shapes.
- See phase spec: `docs/specs/2026-10-31T0154Z_genese-tactile-grid-and-harmonic-matrix-ui/SPEC.md`.

## Acceptance criteria
- [ ] Implement `PitchClassSet::complement(&self) -> PitchClassSet` returning the inverted bitmask (`!self.0 & 0x0FFF`).
- [ ] Implement `render_dodecagram_panel()` in `src/ui.rs`, displaying a 12-point circular or compact clock diagram:
  - 12 chromatic degrees positioned around a perimeter: 12 (C), 1 (C#), ..., 11 (B).
  - Active played pitch classes rendered as illuminated filled nodes (`●`).
  - Inactive pitch classes rendered as muted open nodes (`○` or `·`).
  - Visual lines/chords connecting active nodes when space permits.
- [ ] Display the harmonic complement's pitch classes textually (e.g. `Complement (8): {C#, D, F, ...}`).
- [ ] Layout collapses gracefully when horizontal width is restricted (`< 100` columns).

## Interface contract
```rust
// In src/theory.rs

impl PitchClassSet {
    #[must_use]
    pub fn complement(&self) -> Self;
    pub fn pitch_class_names(&self) -> Vec<&'static str>;
}

// In src/ui.rs

pub fn render_dodecagram_panel(frame: &mut Frame, area: Rect, app: &App, pc_set: PitchClassSet);
```

## Boundaries
- **Touches:**
  - `src/theory.rs`
  - `src/ui.rs`
- **Does NOT touch:**
  - `src/synth.rs`
  - `src/midi.rs`
  - `src/app.rs`

## Verification
```bash
cargo test
```
