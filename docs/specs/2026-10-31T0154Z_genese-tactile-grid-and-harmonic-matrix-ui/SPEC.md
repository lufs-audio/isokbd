# SPEC — `isokbd` Phase 2: Genèse Tactile Grid & Harmonic Matrix UI

**Status:** Ready for Review  
**Owner:** Daniel Ramirez  
**Phase folder:** `docs/specs/2026-10-31T0154Z_genese-tactile-grid-and-harmonic-matrix-ui/`  
**Target Repo:** `lufs-audio/isokbd`  
**Related Repos:** `lufs-audio/genese-dodecaphonique-assets`, `lufs-audio/lrex`, `lufs-audio/apho`

---

## 1. Problem Statement

`isokbd` is an isomorphic MIDI keyboard instrument in Rust with low-latency PipeWire streaming, polyphonic Kitty key-release tracking, Glicol synthesis, and live chord recognition. However, its visual presentation in the terminal currently resembles a basic debugging monitor rather than an illuminated musical instrument:

1. **Utilitarian Keyboard Rendering:** The QWERTY keyboard rows are drawn as flat text brackets: `[ 1:Eb4 ] [ Q:Bb3 ]`. There is no visual sense of physical pad depth, tactile key boundaries, or real-time illumination gradients.
2. **Untapped Theory Visuals (The Genèse Connection):** In `genese-dodecaphonique-assets`, pitch was established as data: **a 12-bit vector field**. Isomorphic keyboards (All-4ths, Wicki-Hayden, Jankó) are physical embodiments of this principle because harmonic relationships remain geometrically invariant anywhere on the grid. Yet `isokbd`'s chord panel merely prints two flat strings: `Chord: C Maj7` and `Active Notes: C3, E3, G3, B3`, discarding the geometric symmetries, interval barcodes, and complementary modal antipodes that the music theory naturally possesses.
3. **Monochrome / Basic ANSI Palette:** Active, root, and sustained notes use basic primary terminal colors (Yellow, Cyan, Green) rather than the cohesive, semantic palettes developed for the LUFS audio ecosystem (`Genese Lufs` charcoal/teal/gold and `Genese DR` warm-black/gold/plum).

---

## 2. Goals & Constraints

### Goals
- **Tactile Pad Grid:** Upgrade keyboard rendering from flat ASCII strings to tactile, box-drawn or sub-cell illuminated pads (`┌───┐`, `│Z C│`, `└───┘`) that visually light up when played, ring when sustained, and prominently highlight modal roots.
- **12-Bit Pitch Vector & Interval Barcode:** Model active harmony in `theory.rs` as a 12-bit pitch-class set (`PitchClassSet`). Display a 12-step interval barcode widget (`■ □ □ □ ■ □ □ ■ □ □ □ ■`) showing semitone intervals, chord density (`k / 12`), and modal formula.
- **Dodecagram & Complementarity Ring:** Render a 12-point geometric polygon / circular pitch-class representation showing chord geometry (triangles for triads, squares for diminished 7ths, hexagons for whole-tone) and its exact Genèse harmonic complement (`12 - k`).
- **Semantic Genèse Color Themes:** Add runtime theme support with 24-bit TrueColor palettes (`GeneseLufs`, `GeneseDr`, and `ClassicAnsi`), switchable on the fly.

### Constraints
- **Zero Audio & Latency Impact:** TUI enhancements must remain strictly non-blocking. Audio synthesis (Glicol) and MIDI event emission run on separate worker threads and must never stutter or drop frames due to UI rendering.
- **Responsive Layout:** The grid and harmonic widgets must adapt cleanly across terminal dimensions: full expanded display on 120x35+ terminals, and clean, unclipped compact fallback on standard 80x24 windows.

---

## 3. Ecosystem References

- **`lufs-audio/genese-dodecaphonique-assets`:** Direct theoretical and visual foundation:
  - *The 12-Bit Field:* Pitch classes as bitmasks `0..11`.
  - *Asset Classes:* The complementarity ring (`k | 12 - k`), the interval cell barcode, and the dodecagram (injective pitch-class polygon).
  - *Color Palettes:* LUFS (`#111111` ground, `#78BEBA` teal, `#D9A23D` gold, `#C4457E` rose) and DR (`#1A1A18` warm-black, `#D9A23D` gold, `#9E7386` plum).
- **`lufs-audio/lrex`:** Terminal audio recorder pairing with `isokbd` for zero-xrun audio + MIDI take capture.
- **`lufs-audio/apho`:** Precision screen perception tool capable of capturing `isokbd` harmonic chord diagrams into Markdown notes.

---

## 4. Architectural Approach

```
┌────────────────────────────────────────────────────────────────────────┐
│                          isokbd Ratatui Frame                          │
├────────────────────────────────────────────────────────────────────────┤
│ Header: Mode | Root | Velocity | Sustain | Synth Engine | MIDI Port    │
├────────────────────────────────────────────────────────────────────────┤
│                      Tactile Isomorphic Key Grid                       │
│    ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐                   │
│    │ 1 Eb4 │ │ 2 E 4 │ │ 3 F 4 │ │ 4 F#4 │ │ 5 G 4 │ ...               │
│    └───────┘ └───────┘ └───────┘ └───────┘ └───────┘                   │
│      ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐                 │
│      │ Q Bb3 │ │ W B 3 │ │ E C 4 │ │ R C#4 │ │ T D 4 │ ...             │
│      └───────┘ └───────┘ └───────┘ └───────┘ └───────┘                 │
├────────────────────────────────────┬───────────────────────────────────┤
│    Harmonic & Chord Detail Panel   │     Dodecagram / Pitch Vector     │
│  Chord:   C Maj7                   │          [12] C   ●───● E [4]     │
│  Degree:  Root (0)                 │              \   │   /            │
│  Formula: [4, 3, 4] • Density: 4/12│               \  │  /             │
│  Barcode: ■ □ □ □ ■ □ □ ■ □ □ □ ■  │          [11] B  ●───● G [7]      │
│  Complement: {C#, D, D#, F, ...}   │          Injective Dodecagram     │
└────────────────────────────────────┴───────────────────────────────────┘
```

### 4.1 Theoretical Kernel (`src/theory.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PitchClassSet(pub u16);

impl PitchClassSet {
    pub fn from_midi_notes(notes: &[u8]) -> Self;
    pub fn contains(&self, pc: u8) -> bool;
    pub fn count(&self) -> u8; // k of 12
    pub fn complement(&self) -> Self; // (12 - k)
    pub fn interval_formula(&self) -> Vec<u8>;
    pub fn barcode_string(&self) -> String; // "■ □ □ □ ■ □ □ ■ □ □ □ ■"
}
```

### 4.2 Pad Box Layout (`src/ui.rs`)
Keys render as structured cells with separate border, label, and note spans:
```text
┌─────┐
│1 Eb4│
└─────┘
```
Compact terminals collapse padding to `[ 1:Eb4 ]` automatically based on `area.width`.

---

## 5. Done Criteria

- [ ] Isomorphic keyboard grid renders using structured pad cells with distinct resting, root, active-held, and sustained visual states.
- [ ] Active chords display the Genèse 12-step interval barcode (`■ □ ...`) and degree density ratio (`k / 12`).
- [ ] Dodecagram / circular pitch-class widget renders geometric chord shapes and identifies the modal complement.
- [ ] Three selectable color palettes (`GeneseLufs`, `GeneseDr`, `ClassicAnsi`), toggleable via `T` key during runtime.
- [ ] Zero audio dropouts or MIDI latency regression confirmed during continuous chord playing.
- [ ] All unit tests pass, and new tests verify `PitchClassSet` arithmetic, formula calculations, and complement symmetry.

---

## 6. Units of Work

- **Unit 01 (`units/01-tactile-pad-grid-rendering.md`):** Multi-line box-drawn pad cell rendering and illumination states in `src/ui.rs`.
- **Unit 02 (`units/02-pitch-class-vector-and-interval-barcode.md`):** `PitchClassSet` 12-bit vector, interval formulas, and interval barcode widget in `src/theory.rs` and `src/ui.rs`.
- **Unit 03 (`units/03-complementarity-dodecagram-widget.md`):** 12-point circular dodecagram widget rendering chord polygons and modal complements.
- **Unit 04 (`units/04-genese-theme-engine-and-shortcuts.md`):** `GeneseLufs` and `GeneseDr` TrueColor themes and runtime theme toggle in `src/app.rs`.
