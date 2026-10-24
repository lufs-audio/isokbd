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
- **Built-in Glicol Synthesizer**: Embedded, zero-dependency polyphonic synth powered by the `glicol` Rust audio engine and `cpal`. Features a 250ms smooth acoustic decay ("damper fall") and dynamic anti-clipping gain scaling right out of the box.
- **Momentary Sustain Pedal**: Hold `Space` down like a physical piano pedal to tap-to-stack notes and build large chords, bypassing standard keyboard matrix ghosting. Releasing `Space` releases all sustained voices.
- **Two-Basis Hex Lattice Engine**: Exact geometric parity with Hexboard, supporting Wicki-Hayden, Tonnetz, Gerhard, Fourths, Jankó, and Piano Row.
- **Zero-Panic Rust**: Built with strict monadic error handling (`unwrap_used = "deny"`).

## Controls

| Key | Action |
|---|---|
| **Any QWERTY Key** | Play notes (polyphonic, illuminated on grid) |
| **`Space` (Hold)** | **Sustain Pedal**: Hold down to tap-to-stack chords; release to damp |
| **`Enter`** | Toggle Built-in Glicol Synth Mute (ON / OFF) |
| **`Tab`** | Cycle layout mode (Wicki-Hayden $\rightarrow$ Tonnetz $\rightarrow$ Gerhard $\rightarrow$ Fourths $\rightarrow$ Jankó $\rightarrow$ Piano Row) |
| **`↑` / `↓`** | Octave Up / Down ($\pm 12$ semitones) |
| **`←` / `→`** | Transpose Root Up / Down ($\pm 1$ semitone) |
| **`PgUp` / `PgDn`** | Adjust MIDI Velocity and Synth Volume ($\pm 10$) |
| **`Esc` or `Ctrl+C`** | Quit cleanly and reset all notes |

## CLI Options

```bash
isokbd               # Starts with both MIDI output and the built-in Glicol synth enabled
isokbd --no-synth    # Starts in MIDI-only mode (internal audio engine disabled)
```

## Playing Chords in Wicki-Hayden Mode (Default)

Wicki-Hayden arranges whole steps horizontally (`+2` semitones East) and fifths diagonally (`+7` semitones North-East):
- **Major Triad**: Root (`Z` = C3) + Major 3rd (`C` = E3, 2 keys right) + 5th (`S` = G3, 1 key up-right)
- **Minor Triad**: Root (`Z` = C3) + Minor 3rd (`A` = Eb3 / F3 depending on voicing) + 5th (`S` = G3)
- **Octave**: Always **Up 2 rows, Right 1 key** (e.g. `Z` $\rightarrow$ `W` = C3 $\rightarrow$ C4)
- **Sustain Stacking**: Hold **Space** down with your thumb, tap `Z`, tap `S`, tap `C`, tap `W` — all 4 notes stay ringing cleanly! Release **Space** to damp them all.

## License

MIT OR Apache-2.0
