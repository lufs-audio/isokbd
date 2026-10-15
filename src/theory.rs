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
}
