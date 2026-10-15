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

You can switch layouts on the fly at any time by pressing **`Tab`**.

### A. All-4ths Mode (Default)
Tuned identically to a 4-string bass, Chapman Stick, or an all-fourths guitar (`E-A-D-G-C-F`).

- **Horizontal (same row)**: $+1$ semitone per key to the right.
- **Vertical (up one row)**: $+5$ semitones (a **Perfect 4th**) directly above.

```text
[Row 3: Numbers]  [1]  [2]  [3]  [4]  [5]  [6]  [7]  [8]  [9]  [0]  [-]  [=]
                   Eb   E    F    F#   G    G#   A    Bb   B    C    C#   D
[Row 2: QWERTY]     [Q]  [W]  [E]  [R]  [T]  [Y]  [U]  [I]  [O]  [P]  [[]  []]
                     Bb   B    C    C#   D    D#   E    F    F#   G    G#   A
[Row 1: Home]         [A]  [S]  [D]  [F]  [G]  [H]  [J]  [K]  [L]  [;]  [']
                       F    F#   G    G#   A    Bb   B    C    C#   D    D#
[Row 0: Bottom]         [Z]  [X]  [C]  [V]  [B]  [N]  [M]  [,]  [.]  [/]
                         C    C#   D    D#   E    F    F#   G    G#   A
```

#### Chord Shapes in All-4ths
Because the grid is strictly regular, your finger shapes **never change**:

* **Major Triad (Root, 3rd, 5th)**:
  - Root: `(row, col)` (e.g. `Z` = C3)
  - Major 3rd: `(row + 1, col - 1)` (e.g. `D` = E3)
  - 5th: `(row + 1, col + 2)` (e.g. `H` = G3)
  - *Finger shape*: A compact upward triangle `Z - D - H`.
* **Minor Triad (Root, b3, 5th)**:
  - Root: `(row, col)` (e.g. `Z` = C3)
  - Minor 3rd: `(row + 1, col - 2)` (e.g. `S` = Eb3)
  - 5th: `(row + 1, col + 2)` (e.g. `H` = G3)
  - *Finger shape*: `Z - S - H`.
* **Octave**:
  - Always **Up 2 rows, Right 2 keys** (e.g. `Z` $\rightarrow$ `E`, or `A` $\rightarrow$ `T`).
* **Minor 7th**:
  - Always **Directly Up 2 rows** (e.g. `Z` $\rightarrow$ `Q`).
* **Major 7th**:
  - Always **Up 2 rows, Right 1 key** (e.g. `Z` $\rightarrow$ `W`).

### B. Jankó Mode
The classic 19th-century keyboard invention by Paul von Jankó:
- Alternating whole-tone scales offset by a half-step.
- Row 0 and Row 2 play whole-tone scale A (`C, D, E, F#, G#, A#`).
- Row 1 and Row 3 play whole-tone scale B (`C#, D#, F, G, A, B`).

### C. Wicki-Hayden Mode
A hexagonal harmonic table layout:
- Horizontal movement steps by whole-tones ($+2$ semitones).
- Diagonal movement steps by 4ths ($+5$) and 5ths ($+7$).

---

## 3. Keyboard Controls

| Key | Action |
|---|---|
| **Any Musical Key** | Play notes (polyphonic, illuminated on the grid) |
| **`↑` / `↓`** | Octave Up / Down ($\pm 12$ semitones) |
| **`←` / `→`** | Transpose Root Up / Down ($\pm 1$ semitone) |
| **`Tab`** | Cycle Layout Mode (All-4ths $\rightarrow$ Jankó $\rightarrow$ Wicki-Hayden) |
| **`Space`** | Toggle Sustain mode (ON / OFF) |
| **`PgUp` / `PgDn`** | Adjust MIDI Velocity ($\pm 10$) |
| **`Backspace` / `Delete`** | Panic button: silence all sounding notes |
| **`Esc` or `Ctrl+C`** | Clean exit |

---

## 4. Audio & MIDI Routing

`isokbd` sends standard MIDI messages through Linux ALSA and PipeWire.

### With `rtrack`
1. Start `rtrack` in a terminal window:
   ```bash
   rtrack
   ```
2. In another window or multiplexer pane, launch:
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

## 5. Terminal Compatibility & Key Release

In standard Unix terminal input, key releases are historically never transmitted. `isokbd` features a **hybrid release engine**:
1. **Kitty Keyboard Protocol**:
   When running in modern terminals (Ghostty, Foot, Kitty, WezTerm, Alacritty), `isokbd` requests progressive enhancement flags (`REPORT_ALL_KEYS_AS_ESCAPE_CODES`). This delivers genuine, sub-millisecond `KeyRelease` events for every text key.
2. **Multiplexer Inactivity Fallback**:
   When running inside terminal multiplexers (like `herdr` or `tmux`) that do not pass through release escape codes, `isokbd` automatically measures key repetition intervals. When you lift your finger, the repeat stream stops, and the note cleanly releases without double-triggering or sticking.
