# Unit 04 — genese-theme-engine-and-shortcuts

## Objective
Introduce TrueColor theme support with the `GeneseLufs` and `GeneseDr` palettes alongside classic ANSI, and allow runtime theme toggling via keyboard shortcut.

## Context
- `src/ui.rs` currently hardcodes basic ANSI colors (`Color::Yellow`, `Color::Cyan`, `Color::Green`, `Color::DarkGray`).
- In `genese-dodecaphonique-assets`, two canonical palettes are defined:
  - `LUFS`: Ground `#111111`, Lead Teal `#78BEBA`, Gold `#D9A23D`, Extension Rose `#C4457E`.
  - `DR`: Ground Warm Near-Black `#1A1A18`, Lead Gold `#D9A23D`, Extension Plum `#9E7386`.
- Operators should be able to switch palettes at runtime without restarting `isokbd`.
- See phase spec: `docs/specs/2026-10-31T0154Z_genese-tactile-grid-and-harmonic-matrix-ui/SPEC.md`.

## Acceptance criteria
- [ ] Define `Theme` enum in `src/app.rs` with variants: `GeneseLufs`, `GeneseDr`, and `ClassicAnsi`.
- [ ] Implement `ColorPalette` struct providing semantic colors:
  - `ground`: base background
  - `lead`: primary active color (Teal or Gold)
  - `accent`: secondary active color (Gold or Plum)
  - `root`: root key indicator
  - `accidental`: sharp/flat key surface
  - `natural`: natural key surface
  - `sustain`: ringing pedal indicator
- [ ] Bind key `T` (or `F2`) in `src/app.rs` `handle_key_event` to cycle through available themes on the fly.
- [ ] Display current theme in header bar: `│ Theme: Genese LUFS │`.
- [ ] Footer shortcuts bar updated to include `[T] Theme`.
- [ ] Unit tests verify theme cycling and color token mappings.

## Interface contract
```rust
// In src/app.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    GeneseLufs,
    GeneseDr,
    ClassicAnsi,
}

impl Theme {
    pub fn next(&self) -> Self;
    pub fn name(&self) -> &'static str;
    pub fn palette(&self) -> ColorPalette;
}

pub struct ColorPalette {
    pub bg: ratatui::style::Color,
    pub lead: ratatui::style::Color,
    pub accent: ratatui::style::Color,
    pub root: ratatui::style::Color,
    pub natural_fg: ratatui::style::Color,
    pub accidental_fg: ratatui::style::Color,
    pub held_bg: ratatui::style::Color,
    pub held_fg: ratatui::style::Color,
    pub sustain_bg: ratatui::style::Color,
}
```

## Boundaries — do NOT touch
- **Touches:**
  - `src/app.rs`
  - `src/ui.rs`
- **Does NOT touch:**
  - `src/synth.rs`
  - `src/midi.rs`
  - `src/layout.rs`

## Output
- Modified: `src/app.rs` (`Theme`, `ColorPalette`, `next`, `name`, `palette`, `T` key handling); `src/ui.rs` (all colors resolved from the active palette instead of hardcoded ANSI).
- Tests added in `src/app.rs` (`#[cfg(test)] mod tests`) for theme cycling and palette contents.
- Commit message: `feat(theme): GeneseLufs / GeneseDr TrueColor palettes with runtime toggle`

## Verification
```bash
cargo test
```
