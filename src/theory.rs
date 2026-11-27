//! Music theory utilities: note names, intervals, and real-time chord detection.

/// Returns the standard pitch name (e.g., "C", "C#", "D") for a given MIDI note number.
#[must_use]
pub fn pitch_class_name(midi_note: u8) -> &'static str {
    const PITCH_CLASSES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let index = (midi_note % 12) as usize;
    PITCH_CLASSES.get(index).copied().unwrap_or("?")
}

/// Returns the full note name with octave (e.g., 60 -> "C4", 48 -> "C3").
#[must_use]
pub fn note_to_name(midi_note: u8) -> String {
    let pitch = pitch_class_name(midi_note);
    let octave = (i16::from(midi_note) / 12) - 1;
    format!("{pitch}{octave}")
}

/// Given a set of active MIDI notes, detects the most likely chord name.
#[must_use]
pub fn detect_chord(notes: &[u8]) -> Option<String> {
    if notes.is_empty() {
        return None;
    }

    if notes.len() == 1 {
        let note = notes.first().copied().unwrap_or_default();
        return Some(format!("Note: {}", note_to_name(note)));
    }

    // Extract sorted unique pitch classes
    let mut pitch_classes: Vec<u8> = notes.iter().map(|&n| n % 12).collect();
    pitch_classes.sort_unstable();
    pitch_classes.dedup();

    // Single pitch class in different octaves
    if pitch_classes.len() == 1 {
        let pc = pitch_classes.first().copied().unwrap_or_default();
        return Some(format!("{} (Octaves)", pitch_class_name(pc)));
    }

    // Try each pitch class as a potential root note
    for &root in &pitch_classes {
        let intervals: Vec<u8> = pitch_classes
            .iter()
            .map(|&p| if p >= root { p - root } else { p + 12 - root })
            .collect();

        let mut sorted_intervals = intervals;
        sorted_intervals.sort_unstable();

        let root_name = pitch_class_name(root);

        // Check common chord formulas
        match sorted_intervals.as_slice() {
            // Dyads
            [0, 7] => return Some(format!("{root_name}5 (Power chord)")),
            [0, 4] => return Some(format!("{root_name} (Major 3rd)")),
            [0, 3] => return Some(format!("{root_name} (Minor 3rd)")),
            [0, 5] => return Some(format!("{root_name} (4th)")),

            // Triads
            [0, 4, 7] => return Some(format!("{root_name} Major")),
            [0, 3, 7] => return Some(format!("{root_name} Minor")),
            [0, 3, 6] => return Some(format!("{root_name} Dim")),
            [0, 4, 8] => return Some(format!("{root_name} Aug")),
            [0, 2, 7] => return Some(format!("{root_name} Sus2")),
            [0, 5, 7] => return Some(format!("{root_name} Sus4")),

            // 7th Chords
            [0, 4, 7, 11] => return Some(format!("{root_name} Maj7")),
            [0, 4, 7, 10] => return Some(format!("{root_name}7 (Dominant)")),
            [0, 3, 7, 10] => return Some(format!("{root_name} Min7")),
            [0, 3, 6, 10] => return Some(format!("{root_name} m7b5 (Half-Dim)")),
            [0, 3, 6, 9] => return Some(format!("{root_name} Dim7")),
            [0, 3, 7, 11] => return Some(format!("{root_name} mMaj7")),

            // 6ths & Add9
            [0, 4, 7, 9] => return Some(format!("{root_name}6")),
            [0, 3, 7, 9] => return Some(format!("{root_name} m6")),
            [0, 2, 4, 7] => return Some(format!("{root_name} Add9")),

            _ => {}
        }
    }

    // Fallback: list of pitch names
    let note_list: Vec<&str> = notes.iter().map(|&n| pitch_class_name(n)).collect();
    Some(note_list.join(" - "))
}

/// A 12-bit pitch-class field (the Genèse 12-bit vector).
///
/// Bit `n` set means pitch class `n` is present, where `0 = C`, `1 = C#`,
/// ..., `11 = B`. Octave information is deliberately discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PitchClassSet(pub u16);

impl PitchClassSet {
    /// Builds a set from MIDI note numbers, folding each note into `0..11`.
    #[must_use]
    pub fn from_midi_notes(notes: &[u8]) -> Self {
        let mut mask = 0u16;
        for &note in notes {
            mask |= 1u16 << (note % 12);
        }
        Self(mask)
    }

    /// Number of distinct active pitch classes, `k` of 12.
    #[must_use]
    pub fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Whether pitch class `pc` (0..11) is present. Out-of-range `pc` is absent.
    #[must_use]
    pub fn contains(&self, pc: u8) -> bool {
        pc < 12 && self.0 & (1u16 << pc) != 0
    }

    /// Semitone distances between consecutive active pitch classes, ascending.
    ///
    /// For a C Maj7 set (`{0, 4, 7, 11}`) this returns `[4, 3, 4]`.
    #[must_use]
    pub fn interval_formula(&self) -> Vec<u8> {
        let active: Vec<u8> = (0..12u8).filter(|&pc| self.contains(pc)).collect();
        active
            .windows(2)
            .filter_map(|pair| {
                let low = pair.first().copied()?;
                let high = pair.get(1).copied()?;
                Some(high - low)
            })
            .collect()
    }

    /// A 12-glyph interval barcode, e.g. `■ □ □ □ ■ □ □ ■ □ □ □ ■`.
    #[must_use]
    pub fn barcode_string(&self) -> String {
        (0..12u8)
            .map(|pc| if self.contains(pc) { "■" } else { "□" })
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// The Genèse harmonic complement: every pitch class *not* present.
    ///
    /// This is the inverted 12-bit field, so `k + (12 - k) == 12` for every set.
    #[must_use]
    pub fn complement(&self) -> Self {
        Self(!self.0 & 0x0FFF)
    }

    /// The pitch-class names present in the set, ascending from C.
    #[must_use]
    pub fn pitch_class_names(&self) -> Vec<&'static str> {
        (0..12u8)
            .filter(|&pc| self.contains(pc))
            .map(pitch_class_name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_names() {
        assert_eq!(note_to_name(60), "C4");
        assert_eq!(note_to_name(48), "C3");
        assert_eq!(note_to_name(69), "A4");
        assert_eq!(note_to_name(54), "F#3");
    }

    #[test]
    fn test_chord_detection() {
        // C Major (C3, E3, G3)
        assert_eq!(detect_chord(&[48, 52, 55]), Some("C Major".to_string()));
        // A Minor (A3, C4, E4)
        assert_eq!(detect_chord(&[57, 60, 64]), Some("A Minor".to_string()));
        // C Maj7 (C3, E3, G3, B3)
        assert_eq!(detect_chord(&[48, 52, 55, 59]), Some("C Maj7".to_string()));
    }

    #[test]
    fn pitch_class_set_bitmask_construction() {
        // C3, E3, G3 -> pitch classes 0, 4, 7 -> bits 0, 4, 7.
        let set = PitchClassSet::from_midi_notes(&[48, 52, 55]);
        assert_eq!(set, PitchClassSet(0b0000_1001_0001));
        assert!(set.contains(0));
        assert!(set.contains(4));
        assert!(set.contains(7));
        assert!(!set.contains(1));

        // Octave duplicates collapse onto a single pitch class.
        assert_eq!(
            PitchClassSet::from_midi_notes(&[48, 60, 72]),
            PitchClassSet(0b1)
        );
        // Empty input is the empty set.
        assert_eq!(PitchClassSet::from_midi_notes(&[]), PitchClassSet(0));
        // Out-of-range pitch classes are never members.
        assert!(!set.contains(12));
    }

    #[test]
    fn pitch_class_set_density_k_of_12() {
        assert_eq!(PitchClassSet::default().count(), 0);
        assert_eq!(PitchClassSet::from_midi_notes(&[48, 52, 55]).count(), 3);
        assert_eq!(PitchClassSet::from_midi_notes(&[48, 52, 55, 59]).count(), 4);
        assert_eq!(
            PitchClassSet::from_midi_notes(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]).count(),
            12
        );
    }

    #[test]
    fn pitch_class_set_interval_formula() {
        // C Major triad {0, 4, 7} -> [4, 3].
        assert_eq!(
            PitchClassSet::from_midi_notes(&[48, 52, 55]).interval_formula(),
            vec![4, 3]
        );
        // C Maj7 {0, 4, 7, 11} -> [4, 3, 4] (the formula shown in the SPEC).
        assert_eq!(
            PitchClassSet::from_midi_notes(&[48, 52, 55, 59]).interval_formula(),
            vec![4, 3, 4]
        );
        // Fewer than two degrees has no interval formula.
        assert!(PitchClassSet::default().interval_formula().is_empty());
        assert!(
            PitchClassSet::from_midi_notes(&[60])
                .interval_formula()
                .is_empty()
        );
    }

    #[test]
    fn pitch_class_set_barcode_rendering() {
        let cmaj7 = PitchClassSet::from_midi_notes(&[48, 52, 55, 59]);
        assert_eq!(cmaj7.barcode_string(), "■ □ □ □ ■ □ □ ■ □ □ □ ■");
        // The barcode always carries exactly twelve glyph cells.
        assert_eq!(cmaj7.barcode_string().split(' ').count(), 12);
        assert_eq!(
            PitchClassSet::default().barcode_string(),
            "□ □ □ □ □ □ □ □ □ □ □ □"
        );
    }

    #[test]
    fn pitch_class_set_complement_symmetry() {
        let cases: [&[u8]; 4] = [&[], &[48], &[48, 52, 55], &[48, 52, 55, 59]];
        for notes in cases {
            let set = PitchClassSet::from_midi_notes(notes);
            let complement = set.complement();
            // k + (12 - k) == 12.
            assert_eq!(u16::from(set.count()) + u16::from(complement.count()), 12);
            // The complement is disjoint and covers the whole field.
            assert_eq!(set.0 & complement.0, 0);
            assert_eq!(set.0 | complement.0, 0x0FFF);
            // Complementing twice is the identity.
            assert_eq!(complement.complement(), set);
        }
    }

    #[test]
    fn known_chord_geometries_have_expected_densities() {
        // Major triad -> 3 active degrees (a triangle).
        assert_eq!(PitchClassSet::from_midi_notes(&[48, 52, 55]).count(), 3);
        // Diminished seventh -> 4 (a square).
        assert_eq!(PitchClassSet::from_midi_notes(&[48, 51, 54, 57]).count(), 4);
        // Whole-tone -> 6 (a hexagon), all in whole steps.
        let whole_tone = PitchClassSet::from_midi_notes(&[48, 50, 52, 54, 56, 58]);
        assert_eq!(whole_tone.count(), 6);
        assert_eq!(whole_tone.interval_formula(), vec![2, 2, 2, 2, 2]);
        // The complement of one whole-tone set is the other whole-tone set.
        assert_eq!(whole_tone.complement().count(), 6);
    }

    #[test]
    fn pitch_class_names_ascend_from_c() {
        let set = PitchClassSet::from_midi_notes(&[48, 52, 55, 59]);
        assert_eq!(set.pitch_class_names(), vec!["C", "E", "G", "B"]);
        assert!(PitchClassSet::default().pitch_class_names().is_empty());
    }
}
