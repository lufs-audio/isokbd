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

- [x] Isomorphic keyboard grid renders using structured pad cells with distinct resting, root, active-held, and sustained visual states.
- [x] Active chords display the Genèse 12-step interval barcode (`■ □ ...`) and degree density ratio (`k / 12`).
- [x] Dodecagram / circular pitch-class widget renders geometric chord shapes and identifies the modal complement.
- [x] Selectable color palettes toggleable at runtime — **amended**: four palettes (`Catppuccin` default, `GeneseLufs`, `GeneseDr`, `ClassicAnsi`) cycled with `F2` / `Ctrl+T`, because plain `T` is a playable pad. See §7.
- [ ] Zero audio dropouts or MIDI latency regression confirmed during continuous chord playing. — **argued, not measured**: the change is render-only and audio/MIDI still run on their own threads, but no benchmark or dropout harness exists. See §7.
- [x] All unit tests pass, and new tests verify `PitchClassSet` arithmetic, formula calculations, and complement symmetry. — 51 tests (40 unit + 11 integration), 0 ignored.

---

## 6. Units of Work

- **Unit 01 (`units/01-tactile-pad-grid-rendering.md`):** Multi-line box-drawn pad cell rendering and illumination states in `src/ui.rs`.
- **Unit 02 (`units/02-pitch-class-vector-and-interval-barcode.md`):** `PitchClassSet` 12-bit vector, interval formulas, and interval barcode widget in `src/theory.rs` and `src/ui.rs`.
- **Unit 03 (`units/03-complementarity-dodecagram-widget.md`):** 12-point circular dodecagram widget rendering chord polygons and modal complements.
- **Unit 04 (`units/04-genese-theme-engine-and-shortcuts.md`):** `GeneseLufs` and `GeneseDr` TrueColor themes and runtime theme toggle in `src/app.rs`.

---

## 7. Status — shipped, with deviations recorded

Implemented on `ciani/phase2-genese-tactile-grid` (PR #5). The contracts above are left as
written — they are the plan, not a changelog. This section records where the implementation
deliberately departs from them, so nothing is silently renegotiated.

### Done criteria, as measured

| Criterion | State | Evidence |
|---|---|---|
| Tactile pad cells, five illumination states | met | `ui::tests::pad_state_selection_and_precedence`, `render_smoke_across_terminal_sizes` |
| Barcode + `k/12` density | met | `theory::tests::*`, `tests/barcode_widget.rs` |
| Dodecagram + modal complement | met | `ui::tests::dodecagram_*`, `layout_regressions::dodecagram_*` |
| Runtime palette cycling | met (amended, see D1) | `app::tests::*theme*` |
| Zero audio dropouts / latency regression | **not measured** | render-only change; no harness exists |
| Tests pass, new theory tests | met | 51 passing, 0 ignored |

### Unit 01's performance criterion is NOT met

> "Ensure rendering is 100% non-allocating in hot loop where possible, preserving 60+ FPS
> terminal redraw without stuttering."

**Not met, and left unticked in Unit 01.** The render path still allocates per cell
(`format!` + owned `String`) and builds a `Vec` per row; the themed-span `PadCell<'a>` shape
hands out `&str` for the note name but the composed cell label has to be owned. No frame-rate
benchmark was run, so 60+ FPS is not claimed either. A zero-alloc rewrite is a separate pass.

### Deviations

- **D1 — Theme set and default.** Unit 04 specifies three palettes with `#[default] GeneseLufs`.
  Shipped: four (`Catppuccin` added and made the default, `GeneseLufs`, `GeneseDr`,
  `ClassicAnsi`). The `Theme`/`ColorPalette` interface is unchanged.
- **D2 — `T` is not a UI key.** Unit 04 asks for `T`; plain `t` is a playable pad (`B3`) in every
  layout. Shipped: `F2` and `Ctrl+T` cycle the palette, and the footer shows `[F2] Theme`.
- **D3 — Pad expansion is gated on height AND width.** Unit 01 ties expansion to
  `height >= 14`; SPEC §4.2 collapses to compact on `width`. Shipped: expanded only when
  `height >= 14 && width >= 92` (12 keys × 7 + 6 stagger + 2), so a tall-but-narrow terminal
  cannot clip the grid.
- **D4 — `interval_formula` omits the closing interval.** `[0,4,7,11]` returns `[4, 3, 4]`,
  matching the example in Unit 02, not `[4,3,4,1]`.
- **D5 — Integration tests reach into the crate by path.** `isokbd` is binary-only, so
  `tests/barcode_widget.rs` uses `#[path = "../src/theory.rs"]`; the clean fix (`src/lib.rs` +
  `main.rs` split) touches `main.rs`, outside the unit's Touches list.
- **D6 — Label toggle was added.** `F3` / `Ctrl+L` hides grid lettering. Not in any unit
  contract; needed to make the "harmonic field" reading of the pad grid usable.
