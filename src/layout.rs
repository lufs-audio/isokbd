//! Isomorphic coordinate systems and key-to-note mapping.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Each row is a Perfect 4th (+5 semitones) above the row below it.
    /// Columns advance by +1 semitone. Identical to bass/Chapman Stick.
    AllFourths,
    /// Traditional whole-tone alternating Jankó layout.
    Janko,
    /// Harmonic table / Wicki-Hayden layout (whole steps horizontal, 4ths/5ths diagonal).
    WickiHayden,
}

impl LayoutMode {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::AllFourths => "All-4ths (Bass/Stick)",
            Self::Janko => "Jankó (Alternating)",
            Self::WickiHayden => "Wicki-Hayden (Hex)",
        }
    }

    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::AllFourths => Self::Janko,
            Self::Janko => Self::WickiHayden,
            Self::WickiHayden => Self::AllFourths,
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

/// Calculates the MIDI note for a given row and column in the selected layout mode.
#[must_use]
pub fn calculate_midi_note(
    row: usize,
    col: usize,
    root_note: u8,
    mode: LayoutMode,
) -> Option<u8> {
    let offset = match mode {
        LayoutMode::AllFourths => {
            let row_offset = (row as i32) * 5;
            let col_offset = col as i32;
            row_offset + col_offset
        }
        LayoutMode::Janko => {
            // Alternating whole tone scales
            let row_offset = match row {
                0 => 0,
                1 => 1,
                2 => 12,
                3 => 13,
                _ => (row as i32) * 6,
            };
            let col_offset = (col as i32) * 2;
            row_offset + col_offset
        }
        LayoutMode::WickiHayden => {
            // Cols are whole steps (+2), rows step by 4ths (+5)
            let row_offset = (row as i32) * 5;
            let col_offset = (col as i32) * 2;
            row_offset + col_offset
        }
    };

    let total = i32::from(root_note) + offset;
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
    fn test_all_fourths_coordinates() {
        // Root = C3 (48)
        let root = 48;
        // Bottom row Z (0, 0) should be C3 (48)
        assert_eq!(calculate_midi_note(0, 0, root, LayoutMode::AllFourths), Some(48));
        // Bottom row C (0, 2) should be D3 (50)
        assert_eq!(calculate_midi_note(0, 2, root, LayoutMode::AllFourths), Some(50));
        // Home row A (1, 0) should be F3 (48 + 5 = 53)
        assert_eq!(calculate_midi_note(1, 0, root, LayoutMode::AllFourths), Some(53));
        // Upper row Q (2, 0) should be Bb3 (48 + 10 = 58)
        assert_eq!(calculate_midi_note(2, 0, root, LayoutMode::AllFourths), Some(58));
        // Number row 1 (3, 0) should be Eb4 (48 + 15 = 63)
        assert_eq!(calculate_midi_note(3, 0, root, LayoutMode::AllFourths), Some(63));

        // Up 2 rows, right 2 keys: (0,0) -> (2,2) should be exact octave (48 + 10 + 2 = 60 / C4)
        assert_eq!(calculate_midi_note(2, 2, root, LayoutMode::AllFourths), Some(60));
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
