# isokbd User Guide

`isokbd` turns your physical computer typing keyboard into a polyphonic, isomorphic MIDI performance instrument inside the terminal.

---

## 1. The Core Idea: Isomorphic Music on QWERTY

On a traditional piano keyboard:
- The interval between adjacent physical keys varies depending on black and white keys (e.g. E to F is a half-step, while C to D is a whole-step).
- Because intervals are uneven, you have to memorize **12 different physical finger shapes** for the exact same chord.

On an **isomorphic layout**:
- Musical intervals map directly to **geometric physical vectors** (direction and distance).
- A major triad has the **exact same physical shape** everywhere on the keyboard.
- To transpose to a new key, you simply move that exact finger shape to a different starting key.

---

## 2. Layout Modes

Press **`Tab`** to cycle layouts at any time. Every layout is one `(right, up_right)` interval pair pushed through a single formula:

```text
note = root + (col × right) + (row × (up_right − right))
```

| `Tab` order | Mode | Right ($\rightarrow$) | Up-right ($\nearrow$) | Straight up ($\uparrow$) | Root on entry |
|---|---|---|---|---|---|
| 1 | Wicki-Hayden (Hex) | +2 (M2) | +7 (P5) | +5 (P4) | `F2` (41) |
| 2 | Tonnetz (Harmonic Table) | +4 (M3) | +7 (P5) | +3 (m3) | `C2` (36) |
| 3 | Gerhard | +3 (m3) | +7 (P5) | +4 (M3) | `C2` (36) |
| 4 | All-4ths (Bass / Stick) | +5 (P4) | +1 (m2) | −4 | `A1` (33) |
| 5 | Jankó (Alternating) | +2 (M2) | +1 (m2) | −1 | `G2` (43) |
| 6 | Piano Row (Chromatic) | +1 (m2) | +5 (P4) | +4 (M3) | `A2` (45) |

Each mode also resets the root to its own default when you `Tab` into it (printed in the header as `Root:`); `←`/`→` transpose from there.

### Default mode: Wicki-Hayden

With the root at `F2`, the grid reads:

| Row | Keys | Notes |
|---|---|---|
| Numbers | `1 2 3 4 5 6 7 8 9 0 - =` | `G#3` `A#3` `C4` `D4` `E4` `F#4` `G#4` `A#4` `C5` `D5` `E5` `F#5` |
| Upper | `Q W E R T Y U I O P [ ]` | `D#3` `F3` `G3` `A3` `B3` `C#4` `D#4` `F4` `G4` `A4` `B4` `C#5` |
| Home | `A S D F G H J K L ; '` | `A#2` `C3` `D3` `E3` `F#3` `G#3` `A#3` `C4` `D4` `E4` `F#4` |
| Bottom | `Z X C V B N M , . /` | `F2` `G2` `A2` `B2` `C#3` `D#3` `F3` `G3` `A3` `B3` |

`C3` lands on `S`, so **C major is `S` + `F` + `E`**. Intervals are constant vectors:

| Interval | Δrows | Δcols | From `S` (`C3`) | Sounds |
|---|---|---|---|---|
| Minor 3rd (+3) | +1 | -1 | `Q` | `D#3` |
| Major 3rd (+4) | +0 | +2 | `F` | `E3` |
| Perfect 4th (+5) | +1 | +0 | `W` | `F3` |
| Perfect 5th (+7) | +1 | +1 | `E` | `G3` |
| Minor 7th (+10) | +2 | +0 | `2` | `A#3` |
| Major 7th (+11) | +1 | +3 | `T` | `B3` |
| Octave (+12) | +2 | +1 | `3` | `C4` |

Move the whole shape to any other root and you have transposed the chord.

### Other layouts, in one line each

- **Tonnetz**: major 3rds run horizontally, fifths up-right, minor 3rds straight up. The classic harmonic table.
- **Gerhard**: minor 3rds horizontal, fifths up-right, major 3rds straight up.
- **All-4ths**: tuned like a 4-string bass / Chapman Stick — fourths horizontal, semitones up-right. Note that straight up is *down* 4 semitones, so vertical finger shapes are not fourths.
- **Jankó**: alternating whole-tone rows offset by a half-step (rows 0/2 and 1/3 carry the two whole-tone scales).
- **Piano Row**: semitones horizontal, fourths up-right; the most literal "piano row" mapping.

---

## 3. The Interface

A real frame at 80×24 — note the compact `S:C3` pads, the truncated header, and the compact
dodecagram clock. At 92×14 or larger the grid renders as box pads and the wheel as a ring
(the README shows the 100×34 form).

```text
╭──────────────────────────────────────────────────────────────────────────────╮
│ ISOKBD │Theme: Catppuccin Mocha│Mode: Wicki-Hayden (Hex)│Root: F2 (41)       │
╰──────────────────────────────────────────────────────────────────────────────╯
╭ Isomorphic QWERTY Grid ──────────────────────────────────────────────────────╮
│   1:G#3 2:A#3 3:C4  4:D4  5:E4  6:F#4 7:G#4 8:A#4 9:C5  0:D5  -:E5  =:F#5    │
│                                                                              │
│     Q:D#3 W:F3  E:G3  R:A3  T:B3  Y:C#4 U:D#4 I:F4  O:G4  P:A4  [:B4  ]:C#5  │
│                                                                              │
│       A:A#2 S:C3  D:D3  F:E3  G:F#3 H:G#3 J:A#3 K:C4  L:D4  ;:E4  ':F#4      │
│                                                                              │
│         Z:F2  X:G2  C:A2  V:B2  B:C#3 N:D#3 M:F3  ,:G3  .:A3  /:B3           │
│                                                                              │
╰──────────────────────────────────────────────────────────────────────────────╯
╭──────────────────────────────────────────────╮╭ Dodecagram · Root F2 ────────╮
│Chord: C Major                                ││                              │
│Formula: [4, 3]  •  Density: 3/12             ││12●   1○   2○   3○   4●   5○  │
│Barcode: ■ □ □ □ ■ □ □ ■ □ □ □ □              ││ 6○   7●   8○   9○  10○  11○  │
│Active:  C3, E3, G3                           ││Complement (9): {C#, D, D#, …}│
│                                              ││                              │
│                                              ││                              │
╰──────────────────────────────────────────────╯╰──────────────────────────────╯
╭──────────────────────────────────────────────────────────────────────────────╮
│      [↑/↓] Octave   [Tab] Layout   [F3] Labels   [F2] Theme   [Esc] Quit     │
╰──────────────────────────────────────────────────────────────────────────────╯
```

The screen is four bands, top to bottom.

### Header

`ISOKBD │ Theme │ Mode │ Root │ Vel │ Pedal │ Synth │ Port`

Fields are added in that priority order and dropped a whole field at a time when the terminal is too narrow, so a value is never sliced in half. `Theme` and `Mode` are always shown if anything is.

### Pad grid

Each key is a pad whose **border colour carries its illumination state**:

| State | Meaning |
|---|---|
| Resting natural | in scale/key surface, not sounding |
| Resting accidental | sharp/flat at rest, dimmer text |
| Root | the current root note, marked with `◆` when labels are hidden |
| Held | physically down right now (`█`) |
| Sustained | no longer held, still ringing under the pedal (`▓`) |

There are two pad geometries plus a hidden-label variant:

- **Expanded** (box pads, `┌─────┐` / `│1 Eb4│` / `└─────┘`) whenever the grid area is at least **92 columns wide and 14 rows tall** — the width is `12 keys × 7 + 6 stagger + 2`, so it only appears on a genuinely large terminal.
- **Compact** (single line) otherwise, at three densities chosen by the space each key gets: `[1:Eb4]` (widest), `1:Eb4`, then bare `1`.
- **`F3` / `Ctrl+L`** hides key/note lettering. Box pads become empty boxes and compact pads collapse to a single state glyph, so the grid reads as a harmonic field; border colour still carries the state.

### Chord panel

Four lines about what you are currently holding:

| Line | Example | Meaning |
|---|---|---|
| `Chord:` | `C Major` | detected chord name, or `No notes playing` |
| `Formula:` / `Density:` | `[4, 3]` · `3/12` | semitone deltas between the active pitch classes, and how many of the 12 degrees are sounding |
| `Barcode:` | `■ □ □ □ ■ □ □ ■ □ □ □ □` | a 12-step map from `C` to `B`: filled = sounding |
| `Active:` | `C3, E3, G3` | exact notes, plus the synth's audio device |

The barcode and density are pitch-class based, so octave doublings do not inflate the count.

### Dodecagram panel

A 12-point wheel of the pitch classes with your chord drawn as a polygon across it, titled with the current root. Below the wheel, the **modal complement** — every pitch class you are *not* playing, `Complement (9): {C#, D, …}` — so you can see the symmetric remainder of the 12-tone circle.

On terminals at least 100 columns wide the dodecagram is pinned to a fixed 32-column panel so its proportions stay stable and the chord panel absorbs the extra space; below that the two panels split 60/40 and the wheel collapses into a compact two-line clock read-out.

### Footer

Priority-ordered shortcut hints (`[↑/↓] Octave`, `[Tab] Layout`, `[F3] Labels`, `[F2] Theme`, `[Space] Pedal`, `[Enter] Synth`, `[←/→] Transpose`, `[PgUp/Dn] Vel`), with `[Esc] Quit` reserved up front so it can never be clipped off the right edge.

---

## 4. Themes

Every colour is resolved from the active palette; nothing in the renderer is hardcoded ANSI. Cycle with **`F2`** or **`Ctrl+T`**:

| Theme | Ground | Lead | Accent |
|---|---|---|---|
| **Catppuccin Mocha** (default) | `#1E1E2E` | `#89B4FA` blue | `#CBA6F7` mauve |
| **Genese LUFS** | `#111111` charcoal | `#78BEBA` teal | `#D9A23D` gold |
| **Genese DR** | `#1A1A18` warm near-black | `#D9A23D` gold | `#9E7386` plum |
| **Classic ANSI** | terminal black | cyan | yellow |

`Ctrl+T` exists because plain `T` is a playable pad in every layout — no letter key is ever reserved for UI.

---

## 5. Keyboard Controls

| Key | Action |
|---|---|
| **Any Musical Key** | Play notes (polyphonic, illuminated on the grid) |
| **`Space`** | Momentary sustain pedal: hold it down to stack notes, release to damp |
| **`Enter`** | Toggle the built-in Glicol synth (mute / unmute) |
| **`↑` / `↓`** | Octave Up / Down ($\pm 12$ semitones) |
| **`←` / `→`** | Transpose Root Up / Down ($\pm 1$ semitone) |
| **`Tab`** | Cycle Layout Mode (Wicki-Hayden → Tonnetz → Gerhard → All-4ths → Jankó → Piano Row) |
| **`PgUp` / `PgDn`** | Adjust MIDI Velocity ($\pm 10$) |
| **`F2`** / **`Ctrl+T`** | Cycle color theme |
| **`F3`** / **`Ctrl+L`** | Toggle key/note lettering on the grid |
| **`Backspace` / `Delete`** | Panic button: silence all sounding notes |
| **`Esc` or `Ctrl+C`** | Clean exit |

Changing layout or root sends **All Notes Off** first, so a held shape cannot strand voices in the old key.

---

## 6. Audio & MIDI Routing

`isokbd` sends standard MIDI messages through Linux ALSA and PipeWire.

### With `rtrack`
1. Start `rtrack` in a terminal window:
   ```bash
   rtrack
   ```
2. In another window, launch:
   ```bash
   isokbd
   ```
3. `isokbd` automatically detects and connects to `RTRACK_MIDI_IN` and `Midi Through Port-0`.
4. Play chords on `isokbd`—notes stream directly into `rtrack`'s synth engine.

### With `fluidsynth` (SoundFont GM Synth)
```bash
fluidsynth -a pipewire -m alsa_seq /usr/share/soundfonts/default.sf2
```
`isokbd` will automatically route into FluidSynth via `Midi Through` or direct port discovery.

---

## 7. Terminal Compatibility & Key Release

In standard Unix terminal input, key **releases** are historically never transmitted. `isokbd` does not guess at them — it asks for them:

- On startup it pushes the Kitty Keyboard Protocol progressive enhancement flags `DISAMBIGUATE_ESCAPE_CODES`, `REPORT_ALL_KEYS_AS_ESCAPE_CODES`, and `REPORT_EVENT_TYPES`.
- In a terminal that honours them (`Ghostty`, `Kitty`, `Foot`, `WezTerm`, `Alacritty`) every text key produces a genuine `Press` **and** `Release`, so a note starts on press and stops on release with sub-millisecond latency. OS typematic `Repeat` events are discarded, so a held key cannot stutter.
- **There is no timeout fallback.** `App::tick()` is deliberately a no-op, and `Repeat` events are ignored rather than measured. A note stops only when a real `Release` arrives, when the sustain pedal is lifted, or when you hit `Backspace` / `Delete`.

### Inside a multiplexer

Multiplexers commonly fail to forward the kitty release encoding to the inner program. If you run `isokbd` inside one that does not (e.g. `herdr`, `tmux`), you will see **stuck notes**: the note starts and never stops.

Run `isokbd` in a plain terminal window instead — a new `ghostty` shell rather than a multiplexer pane. If you must run it inside a multiplexer, keep `Backspace` / `Delete` under a finger, and note that the panic key and `Esc` are the only recovery paths.
