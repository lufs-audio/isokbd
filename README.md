# isokbd

**isokbd** is a terminal-native, polyphonic isomorphic MIDI keyboard instrument written in Rust.

It maps your standard QWERTY keyboard onto a staggered 2D musical matrix (All-4ths, Wicki-Hayden, or Jankó), renders an illuminated real-time TUI grid, detects chords on the fly, and streams low-latency MIDI to ALSA / PipeWire (`Midi Through Port-0`).

```text
┌── ISO-QWERTY ──────────────────────────────[ Mode: All-4ths | Root: C3 (48) | Port: Midi Through ]──┐
│                                                                                                      │
│   [ 1:Eb4] [ 2:E 4] [ 3:F 4] [ 4:F#4] [ 5:G 4] [ 6:G#4] [ 7:A 4] [ 8:Bb4] [ 9:B 4] [ 0:C 5] ...     │
│     [ Q:Bb3] [ W:B 3] [ E:C 4] [ R:C#4] [ T:D 4] [ Y:D#4] [ U:E 4] [ I:F 4] [ O:F#4] [ P:G 4] ...     │
│       [ A:F 3] [ S:F#3] [ D:G 3] [ F:G#3] [ G:A 3] [ H:Bb3] [ J:B 3] [ K:C 4] [ L:C#4] [ ;:D 4] ...     │
│         [ Z:C 3] [ X:C#3] [ C:D 3] [ V:D#3] [ B:E 3] [ N:F 3] [ M:F#3] [ ,:G 3] [ .:G#3] [ /:A 3] ...     │
│                                                                                                      │
├── Chord: C Maj7 (C3, E3, G3, B3) ────────────────────────────────── Active Notes: C3, E3, G3, B3 ───┤
│                                                                                                      │
│   [↑/↓] Octave  [←/→] Transpose  [Tab] Layout  [Space] Sustain  [PgUp/PgDn] Velocity  [Esc] Quit    │
└──────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Features

- **True Polyphony & Real KeyRelease**: Leverages the Kitty Keyboard Protocol via `crossterm` for genuine `NoteOn` on press and `NoteOff` on release.
- **Illuminated Keyboard Grid**: Real-time visual feedback with pitch-class coloring (Roots highlighted in Cyan, Naturals in White, Accidentals in Dark Gray, Active keys illuminated in bold Yellow).
- **Live Chord Detection**: Automatically analyzes your active notes and prints the harmonic chord name (e.g. `C Major`, `F# Min7`, `D7`, `A5`, `E mMaj7`).
- **Multiple Isomorphic Layouts**:
  - **All-4ths (Bass / Chapman Stick)**: Rows are $+5$ semitones apart; columns are $+1$ semitone. Geometric chord and scale shapes are completely invariant across keys!
  - **Jankó (Alternating)**: Whole-tone scales alternating by half-steps.
  - **Wicki-Hayden (Harmonic Hex)**: Whole-steps horizontal, 4ths/5ths diagonal.
- **Instant ALSA / PipeWire Routing**: Automatically connects to `Midi Through Port-0` (or creates a virtual port `isokbd`), immediately driving `rtrack`, `fluidsynth`, `scsynth`, or your DAW.
- **Zero-Panic Rust**: Built with strict monadic error handling (`unwrap_used = "deny"`).

## Controls

| Key | Action |
|---|---|
| **Any QWERTY Key** | Play notes (polyphonic, illuminated on grid) |
| **`↑` / `↓`** | Octave Up / Down ($\pm 12$ semitones) |
| **`←` / `→`** | Transpose Root Up / Down ($\pm 1$ semitone) |
| **`Tab`** | Switch layout mode (All-4ths $\rightarrow$ Jankó $\rightarrow$ Wicki-Hayden) |
| **`Space`** | Toggle Sustain mode (ON / OFF) |
| **`PgUp` / `PgDn`** | Adjust MIDI Velocity ($\pm 10$) |
| **`Esc` or `Ctrl+C`** | Quit cleanly and reset all notes |

## Playing Chords in All-4ths Mode

Because every interval is geometric, your hand shapes **never change** when modulating:
- **Major Triad**: Root (`Z`) + Major 3rd (`D`) + 5th (`H`)
- **Minor Triad**: Root (`Z`) + Minor 3rd (`S`) + 5th (`H`)
- **Octave**: Always **Up 2 rows, Right 2 keys** (e.g. `Z` $\rightarrow$ `E`)
- **Minor 7th**: Always **Directly Up 2 rows** (e.g. `Z` $\rightarrow$ `Q`)
- **Major 7th**: Always **Up 2 rows, Right 1 key** (e.g. `Z` $\rightarrow$ `W`)

## License

MIT OR Apache-2.0
