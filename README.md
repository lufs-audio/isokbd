# isokbd

**isokbd** is a terminal-native, polyphonic isomorphic MIDI keyboard instrument written in Rust.

It maps your standard QWERTY keyboard onto a staggered 2D musical lattice (Wicki-Hayden, Tonnetz, Gerhard, All-4ths, Jankó, Piano Row), renders an illuminated real-time TUI, names the chord you are holding, and streams low-latency MIDI to ALSA / PipeWire (`Midi Through Port-0`) or into its embedded Glicol synth.

A real frame, rendered at 100×34 with C major held on `S` + `F` + `E` (Catppuccin Mocha, the default theme):

```text
╭──────────────────────────────────────────────────────────────────────────────────────────────────╮
│ ISOKBD │Theme: Catppuccin Mocha│Mode: Wicki-Hayden (Hex)│Root: F2 (41)│Vel: 100│Pedal: UP        │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
╭ Isomorphic QWERTY Grid ──────────────────────────────────────────────────────────────────────────╮
│                                                                                                  │
│      ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐        │
│      │1 G#3││2 A#3││3 C4 ││4 D4 ││5 E4 ││6 F#4││7 G#4││8 A#4││9 C5 ││0 D5 ││- E5 ││= F#5│        │
│      └─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘        │
│        ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐      │
│        │Q D#3││W F3 ││E G3 ││R A3 ││T B3 ││Y C#4││U D#4││I F4 ││O G4 ││P A4 ││[ B4 ││] C#5│      │
│        └─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘      │
│          ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐           │
│          │A A#2││S C3 ││D D3 ││F E3 ││G F#3││H G#3││J A#3││K C4 ││L D4 ││; E4 ││' F#4│           │
│          └─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘           │
│            ┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐┌─────┐                │
│            │Z F2 ││X G2 ││C A2 ││V B2 ││B C#3││N D#3││M F3 ││, G3 ││. A3 ││/ B3 │                │
│            └─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘└─────┘                │
│                                                                                                  │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
╭──────────────────────────────────────────────────────────────────╮╭ Dodecagram · Root F2 ────────╮
│Chord: C Major                                                    ││        11    12     1        │
│Formula: [4, 3]  •  Density: 3/12                                 ││          ○···●····○          │
│Barcode: ■ □ □ □ ■ □ □ ■ □ □ □ □                                  ││    10  ··   │ ╲─   ··  2     │
│Active:  C3, E3, G3                                               ││      ○·     ╱   ╲─   ○       │
│                                                                  ││   9 ○      ╱      ╲   ○ 3    │
│                                                                  ││      ·    │        ╲─ ·      │
│                                                                  ││      ○·   ╱    ──────●       │
│                                                                  ││    8   ··●───○╱···○·   4     │
│                                                                  ││        7     6      5        │
│                                                                  ││Complement (9): {C#, D, D#, …}│
╰──────────────────────────────────────────────────────────────────╯╰──────────────────────────────╯
╭──────────────────────────────────────────────────────────────────────────────────────────────────╮
│        [↑/↓] Octave   [Tab] Layout   [F3] Labels   [F2] Theme   [Space] Pedal   [Esc] Quit       │
╰──────────────────────────────────────────────────────────────────────────────────────────────────╯
```

## Features

- **True Polyphony & Real KeyRelease**: Leverages the Kitty Keyboard Protocol via `crossterm` for genuine `NoteOn` on press and `NoteOff` on release.
- **Tactile Pad Grid**: Every key renders as a box-drawn pad (`┌─────┐`) with five illumination states — resting natural, resting accidental, root, held, and pedal-sustained. The pad's border carries the state, so `F3` can hide the lettering and leave a clean harmonic field.
- **Responsive Rendering**: Wide terminals get multi-line box pads; narrow or short ones degrade to a single-line `[1:Eb4]` cell, then to `1:Eb4`, then to bare key letters, so nothing is ever clipped off the grid. Header and footer drop whole fields instead of slicing through a value.
- **Harmony Read-out**: The chord panel names the chord (`C Major`), prints its interval formula (`[4, 3]`), the 12-step Genèse barcode (`■ □ □ □ ■ □ □ ■ □ □ □ □`), and the degree density (`k / 12`).
- **Dodecagram**: A 12-point pitch-class wheel that draws the held chord as a polygon and names the modal complement (`12 − k`), collapsing to a compact clock read-out on narrow terminals.
- **Four TrueColor Themes**: Catppuccin Mocha (default), Genese LUFS, Genese DR, and Classic ANSI. Cycle at runtime with `F2` / `Ctrl+T`.
- **Live Chord Detection**: Automatically analyzes your active notes and prints the harmonic chord name (e.g. `C Major`, `F# Min7`, `D7`, `A5`, `E mMaj7`).
- **Six Isomorphic Layouts**:
  - **Wicki-Hayden (default)**: whole steps horizontal, 4ths/5ths diagonal.
  - **Tonnetz (Harmonic Table)**: major 3rds horizontal, fifths up-right.
  - **Gerhard**: minor 3rds horizontal, fifths up-right.
  - **All-4ths (Bass / Chapman Stick)**: fourths horizontal, semitones up-right.
  - **Jankó (Alternating)**: whole-tone rows alternating by half-steps.
  - **Piano Row**: chromatic horizontal, fourths up-right.
- **Instant ALSA / PipeWire Routing**: Automatically connects to `Midi Through Port-0` (or creates a virtual port `isokbd`), immediately driving `rtrack`, `fluidsynth`, `scsynth`, or your DAW.
- **Built-in Glicol Synthesizer**: Embedded, zero-dependency polyphonic synth powered by the `glicol` Rust audio engine and `cpal`. Features a 250ms smooth acoustic decay ("damper fall") and dynamic anti-clipping gain scaling right out of the box.
- **Momentary Sustain Pedal**: Hold `Space` down like a physical piano pedal to tap-to-stack notes and build large chords, bypassing standard keyboard matrix ghosting. Releasing `Space` releases all sustained voices.
- **Two-Basis Hex Lattice Engine**: Exact geometric parity with Hexboard — every layout is one `(right, up_right)` interval pair resolved through the same formula.
- **Zero-Panic Rust**: Built with strict monadic error handling (`unwrap_used = "deny"`).

## Controls

| Key | Action |
|---|---|
| **Any QWERTY Key** | Play notes (polyphonic, illuminated on grid) |
| **`Space` (Hold)** | **Sustain Pedal**: hold down to tap-to-stack chords; release to damp |
| **`Enter`** | Toggle Built-in Glicol Synth Mute (ON / OFF) |
| **`Tab`** | Cycle layout mode (Wicki-Hayden $\rightarrow$ Tonnetz $\rightarrow$ Gerhard $\rightarrow$ All-4ths $\rightarrow$ Jankó $\rightarrow$ Piano Row) |
| **`↑` / `↓`** | Octave Up / Down ($\pm 12$ semitones) |
| **`←` / `→`** | Transpose Root Up / Down ($\pm 1$ semitone) |
| **`PgUp` / `PgDn`** | Adjust MIDI Velocity and Synth Volume ($\pm 10$) |
| **`F2`** / **`Ctrl+T`** | Cycle color theme |
| **`F3`** / **`Ctrl+L`** | Toggle key/note lettering on the grid |
| **`Backspace`** / **`Delete`** | Panic button: silence all sounding notes |
| **`Esc` or `Ctrl+C`** | Quit cleanly and reset all notes |

## CLI Options

```bash
isokbd               # Starts with both MIDI output and the built-in Glicol synth enabled
isokbd --no-synth    # Starts in MIDI-only mode (internal audio engine disabled)
```

## Playing Chords in Wicki-Hayden Mode (Default)

Rows step by a perfect 4th ($+5$ semitones) as you move up, columns by a whole step ($+2$), and the diagonal up-right is a perfect 5th ($+7$). With the default root (`F2`), `C3` sits on `S`:

| Interval | Δrows | Δcols | From `S` (`C3`) | Sounds |
|---|---|---|---|---|
| Minor 3rd (+3) | +1 | -1 | `Q` | `D#3` |
| Major 3rd (+4) | +0 | +2 | `F` | `E3` |
| Perfect 4th (+5) | +1 | +0 | `W` | `F3` |
| Perfect 5th (+7) | +1 | +1 | `E` | `G3` |
| Minor 7th (+10) | +2 | +0 | `2` | `A#3` |
| Major 7th (+11) | +1 | +3 | `T` | `B3` |
| Octave (+12) | +2 | +1 | `3` | `C4` |

So a **C major triad** is `S` + `F` + `E`, and its shape is the same on any `S`-equivalent root. **Sustain Stacking**: hold **`Space`** with your thumb, tap `S`, `F`, `E`, then `3` — all four notes stay ringing; release **`Space`** to damp them all.

## Terminal Requirements

`isokbd` needs a terminal that speaks the Kitty keyboard protocol (`Ghostty`, `Kitty`, `Foot`, `WezTerm`, `Alacritty`) so that key *releases* are delivered. Inside a multiplexer that does not forward them (e.g. `tmux`, or `herdr`), notes will not release — see [`docs/USAGE.md`](docs/USAGE.md#7-terminal-compatibility--key-release) for the workaround.

## Docs

- [`docs/USAGE.md`](docs/USAGE.md) — the user guide: layouts, interface, themes, controls, routing.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — modules, note lifecycle, and the harmony kernel.

## License

MIT OR Apache-2.0
