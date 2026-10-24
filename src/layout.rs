//! Isomorphic coordinate systems and key-to-note mapping.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Wicki-Hayden hexagonal layout (whole-tone East +2, fifth NE +7, fourth NW +5).
    WickiHayden,
    /// Tonnetz / Harmonic Table (major 3rd East +4, fifth NE +7, minor 3rd NW +3).
    Tonnetz,
    /// Gerhard (minor 3rd East +3, fifth NE +7, whole-tone NW +4).
    Gerhard,
    /// Fourths / Bass (fourth East +5, semitone NE +1).
    AllFourths,
    /// Traditional whole-tone alternating Jankó layout.
    Janko,
    /// Piano Row (chromatic East +1, fourth NE +5).
    PianoRow,
}

impl LayoutMode {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::WickiHayden => "Wicki-Hayden (Hex)",
            Self::Tonnetz => "Tonnetz (Harmonic Table)",
            Self::Gerhard => "Gerhard (Min3/5th)",
            Self::AllFourths => "All-4ths (Bass/Stick)",
            Self::Janko => "Jankó (Alternating)",
            Self::PianoRow => "Piano Row (Chromatic)",
        }
    }

    /// Basis intervals: `(right, up_right)`.
    /// `right`: semitones moving 1 step East (col + 1).
    /// `up_right`: semitones moving 1 step North-East (row + 1, col + 1).
    /// Up-Left is derived geometrically: `up_right - right`.
    #[must_use]
    pub const fn intervals(&self) -> (i32, i32) {
        match self {
            Self::WickiHayden => (2, 7),
            Self::Tonnetz => (4, 7),
            Self::Gerhard => (3, 7),
            Self::AllFourths => (5, 1),
            Self::Janko => (2, 1),
            Self::PianoRow => (1, 5),
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::WickiHayden => Self::Tonnetz,
            Self::Tonnetz => Self::Gerhard,
            Self::Gerhard => Self::AllFourths,
            Self::AllFourths => Self::Janko,
            Self::Janko => Self::PianoRow,
            Self::PianoRow => Self::WickiHayden,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyDef {
    pub label: &'static str,
    pub chars: &'static [char],
    #[allow(dead_code)]
    pub row: usize, // 0 = bottom row, 3 = top number row
    pub col: usize, // 0 = leftmost key in that row
}

// Staggered rows representation
pub const ROW_0_BOTTOM: &[KeyDef] = &[
    KeyDef { label: "Z", chars: &['z', 'Z'], row: 0, col: 0 },
    KeyDef { label: "X", chars: &['x', 'X'], row: 0, col: 1 },
    KeyDef { label: "C", chars: &['c', 'C'], row: 0, col: 2 },
    KeyDef { label: "V", chars: &['v', 'V'], row: 0, col: 3 },
    KeyDef { label: "B", chars: &['b', 'B'], row: 0, col: 4 },
    KeyDef { label: "N", chars: &['n', 'N'], row: 0, col: 5 },
    KeyDef { label: "M", chars: &['m', 'M'], row: 0, col: 6 },
    KeyDef { label: ",", chars: &[',', '<'], row: 0, col: 7 },
    KeyDef { label: ".", chars: &['.', '>'], row: 0, col: 8 },
    KeyDef { label: "/", chars: &['/', '?'], row: 0, col: 9 },
];

pub const ROW_1_HOME: &[KeyDef] = &[
    KeyDef { label: "A", chars: &['a', 'A'], row: 1, col: 0 },
    KeyDef { label: "S", chars: &['s', 'S'], row: 1, col: 1 },
    KeyDef { label: "D", chars: &['d', 'D'], row: 1, col: 2 },
    KeyDef { label: "F", chars: &['f', 'F'], row: 1, col: 3 },
    KeyDef { label: "G", chars: &['g', 'G'], row: 1, col: 4 },
    KeyDef { label: "H", chars: &['h', 'H'], row: 1, col: 5 },
    KeyDef { label: "J", chars: &['j', 'J'], row: 1, col: 6 },
    KeyDef { label: "K", chars: &['k', 'K'], row: 1, col: 7 },
    KeyDef { label: "L", chars: &['l', 'L'], row: 1, col: 8 },
    KeyDef { label: ";", chars: &[';', ':'], row: 1, col: 9 },
    KeyDef { label: "'", chars: &['\'', '"'], row: 1, col: 10 },
];

pub const ROW_2_UPPER: &[KeyDef] = &[
    KeyDef { label: "Q", chars: &['q', 'Q'], row: 2, col: 0 },
    KeyDef { label: "W", chars: &['w', 'W'], row: 2, col: 1 },
    KeyDef { label: "E", chars: &['e', 'E'], row: 2, col: 2 },
    KeyDef { label: "R", chars: &['r', 'R'], row: 2, col: 3 },
    KeyDef { label: "T", chars: &['t', 'T'], row: 2, col: 4 },
    KeyDef { label: "Y", chars: &['y', 'Y'], row: 2, col: 5 },
    KeyDef { label: "U", chars: &['u', 'U'], row: 2, col: 6 },
    KeyDef { label: "I", chars: &['i', 'I'], row: 2, col: 7 },
    KeyDef { label: "O", chars: &['o', 'O'], row: 2, col: 8 },
    KeyDef { label: "P", chars: &['p', 'P'], row: 2, col: 9 },
    KeyDef { label: "[", chars: &['[', '{'], row: 2, col: 10 },
    KeyDef { label: "]", chars: &[']', '}'], row: 2, col: 11 },
];

pub const ROW_3_NUMBER: &[KeyDef] = &[
    KeyDef { label: "1", chars: &['1', '!'], row: 3, col: 0 },
    KeyDef { label: "2", chars: &['2', '@'], row: 3, col: 1 },
    KeyDef { label: "3", chars: &['3', '#'], row: 3, col: 2 },
    KeyDef { label: "4", chars: &['4', '$'], row: 3, col: 3 },
    KeyDef { label: "5", chars: &['5', '%'], row: 3, col: 4 },
    KeyDef { label: "6", chars: &['6', '^'], row: 3, col: 5 },
    KeyDef { label: "7", chars: &['7', '&'], row: 3, col: 6 },
    KeyDef { label: "8", chars: &['8', '*'], row: 3, col: 7 },
    KeyDef { label: "9", chars: &['9', '('], row: 3, col: 8 },
    KeyDef { label: "0", chars: &['0', ')'], row: 3, col: 9 },
    KeyDef { label: "-", chars: &['-', '_'], row: 3, col: 10 },
    KeyDef { label: "=", chars: &['=', '+'], row: 3, col: 11 },
];

pub const ALL_ROWS: [&[KeyDef]; 4] = [
    ROW_0_BOTTOM,
    ROW_1_HOME,
    ROW_2_UPPER,
    ROW_3_NUMBER,
];

/// Calculates the MIDI note for a given row and column in the selected layout mode
/// using the two-basis interval formula derived from hexagonal lattice geometry:
/// `note = root + (col * right) + (row * (up_right - right))`
#[must_use]
pub fn calculate_midi_note(
    row: usize,
    col: usize,
    root_note: u8,
    mode: LayoutMode,
) -> Option<u8> {
    let (right, up_right) = mode.intervals();
    let col_offset = (col as i32) * right;
    let row_offset = (row as i32) * (up_right - right);
    let total = i32::from(root_note) + col_offset + row_offset;

    if (0..=127).contains(&total) {
        #[allow(clippy::cast_possible_truncation)]
        Some(total as u8)
    } else {
        None
    }
}

/// Resolves a keyboard character to its grid coordinate `(row, col)`.
#[must_use]
pub fn char_to_coord(c: char) -> Option<(usize, usize)> {
    for (row_idx, row) in ALL_ROWS.iter().enumerate() {
        for key in *row {
            if key.chars.contains(&c) {
                return Some((row_idx, key.col));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wicki_hayden_intervals() {
        // Root = C3 (48)
        let root = 48;
        // Bottom row Z (0, 0) is Root (48 = C3)
        assert_eq!(calculate_midi_note(0, 0, root, LayoutMode::WickiHayden), Some(48));
        // East: X (0, 1) is +2 (50 = D3)
        assert_eq!(calculate_midi_note(0, 1, root, LayoutMode::WickiHayden), Some(50));
        // East x2: C (0, 2) is +4 (52 = E3, Major 3rd!)
        assert_eq!(calculate_midi_note(0, 2, root, LayoutMode::WickiHayden), Some(52));
        // Up-Left: A (1, 0) is +5 (53 = F3, Perfect 4th!)
        assert_eq!(calculate_midi_note(1, 0, root, LayoutMode::WickiHayden), Some(53));
        // Up-Right: S (1, 1) is +7 (55 = G3, Perfect 5th!)
        assert_eq!(calculate_midi_note(1, 1, root, LayoutMode::WickiHayden), Some(55));
        // Octave: W (2, 1) is Up 2 rows, Right 1 col = +12 (60 = C4, Octave!)
        assert_eq!(calculate_midi_note(2, 1, root, LayoutMode::WickiHayden), Some(60));
    }

    #[test]
    fn test_char_resolution() {
        assert_eq!(char_to_coord('z'), Some((0, 0)));
        assert_eq!(char_to_coord('Z'), Some((0, 0)));
        assert_eq!(char_to_coord('a'), Some((1, 0)));
        assert_eq!(char_to_coord('q'), Some((2, 0)));
        assert_eq!(char_to_coord('1'), Some((3, 0)));
        assert_eq!(char_to_coord('='), Some((3, 11)));
        assert_eq!(char_to_coord(' '), None);
    }
}
