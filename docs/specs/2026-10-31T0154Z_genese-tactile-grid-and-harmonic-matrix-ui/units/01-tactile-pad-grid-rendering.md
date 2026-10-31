# Unit 01 — tactile-pad-grid-rendering

## Objective
Upgrade the isomorphic keyboard grid rendering from flat text brackets to multi-line box-drawn pad cells with dynamic illumination states for resting, root, active-held, and sustained notes.

## Context
- `src/ui.rs` `render_keyboard_grid()` currently builds lines by pushing string slices: `format!(" [{:>1} {:<3}]", key.label, note_str)`.
- Keys lack visual depth, pad boundaries, and distinct tactile hierarchy.
- In `genese-dodecaphonique-assets`, the Matrix direction treats keys as discrete, illuminated instrument cells.
- See phase spec: `docs/specs/2026-10-31T0154Z_genese-tactile-grid-and-harmonic-matrix-ui/SPEC.md`.

## Acceptance criteria
- [ ] Render each key as a multi-line structured pad cell using Unicode box characters (`┌───┐`, `│...│`, `└───┘`) when vertical space allows (`grid_area.height >= 14`).
- [ ] Automatically fallback to compact single-line rendering `[ 1:Eb4 ]` when terminal height is constrained.
- [ ] Support distinct visual styles:
  - **Natural resting key:** subtle border with standard text style.
  - **Accidental resting key:** darker contrast background / muted label.
  - **Root note key:** high-visibility border/glyph highlight.
  - **Physically held key:** bold illumination fill with inverted high-contrast text.
  - **Sustained ringing key:** ringing border color with subtle active background.
- [ ] Maintain the existing QWERTY stagger offsets (`[6, 4, 2, 0]`) across all 4 rows.
- [ ] Ensure rendering is 100% non-allocating in hot loop where possible, preserving 60+ FPS terminal redraw without stuttering.

## Interface contract
```rust
// In src/ui.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadVisualState {
    RestingNatural,
    RestingAccidental,
    Root,
    ActiveHeld,
    Sustained,
}

pub struct PadCell<'a> {
    pub key_char: &'a str,
    pub note_name: &'a str,
    pub state: PadVisualState,
}

pub fn render_keyboard_grid(frame: &mut Frame, area: Rect, app: &App);
```

## Boundaries
- **Touches:**
  - `src/ui.rs`
- **Does NOT touch:**
  - `src/synth.rs` (audio generation is independent)
  - `src/midi.rs` (MIDI pipeline is independent)
  - `src/theory.rs`

## Verification
```bash
cargo check --bin isokbd
cargo test
```
