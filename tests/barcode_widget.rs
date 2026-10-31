//! Integration coverage for the Genèse 12-step interval barcode widget.
//!
//! `isokbd` builds as a binary crate, so the self-contained theory kernel is
//! pulled in directly by path rather than through a library target. This
//! exercises the barcode from outside the crate, the way a widget consumer
//! would.

#[path = "../src/theory.rs"]
mod theory;

use theory::PitchClassSet;

#[test]
fn barcode_widget_reflects_active_degrees() {
    let c_maj7 = PitchClassSet::from_midi_notes(&[48, 52, 55, 59]);
    assert_eq!(c_maj7.count(), 4);
    assert_eq!(c_maj7.interval_formula(), vec![4, 3, 4]);
    assert_eq!(c_maj7.barcode_string(), "■ □ □ □ ■ □ □ ■ □ □ □ ■");
}

#[test]
fn barcode_density_ratio_matches_degree_count() {
    let cases: [(&[u8], usize); 4] = [
        (&[], 0),
        (&[48], 1),
        (&[48, 52, 55], 3),
        (&[48, 52, 55, 59], 4),
    ];

    for (notes, expected) in cases {
        let set = PitchClassSet::from_midi_notes(notes);
        assert_eq!(usize::from(set.count()), expected);

        let barcode = set.barcode_string();
        // Always twelve glyph cells...
        assert_eq!(barcode.split(' ').count(), 12);
        // ...with exactly `k` of them illuminated.
        let illuminated = barcode.chars().filter(|&glyph| glyph == '■').count();
        assert_eq!(illuminated, expected);
    }
}
