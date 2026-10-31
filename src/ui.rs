//! Terminal User Interface rendering with Ratatui.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::{App, ColorPalette};
use crate::layout::{ALL_ROWS, calculate_midi_note};
use crate::theory::{PitchClassSet, note_to_name, pitch_class_name};

/// Visual illumination state of a single isomorphic key pad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadVisualState {
    /// A natural key at rest (no illumination).
    RestingNatural,
    /// An accidental (sharp/flat) key at rest.
    RestingAccidental,
    /// A key whose pitch class matches the current modal root.
    Root,
    /// A key that is physically held down right now.
    ActiveHeld,
    /// A key that is still ringing because of the sustain pedal.
    Sustained,
}

/// A single key pad: its QWERTY label, rendered note name, and illumination state.
pub struct PadCell<'a> {
    pub key_char: &'a str,
    pub note_name: &'a str,
    pub state: PadVisualState,
}

/// Stagger offsets (in characters) applied to rows 3, 2, 1, 0 respectively.
const ROW_STAGGERS: [usize; 4] = [6, 4, 2, 0];

/// Width of an expanded box-drawn pad cell (e.g. `┌─────┐`).
const PAD_CELL_WIDTH: usize = 7;

/// Minimum grid height before pads expand into multi-line box cells.
const EXPANDED_MIN_HEIGHT: u16 = 14;

/// Frame width at or above which the bottom panels sit side-by-side.
const WIDE_LAYOUT_MIN_WIDTH: u16 = 100;

/// Inner size required before the dodecagram renders as a full circular ring.
const DODECA_RING_MIN_W: u16 = 40;
const DODECA_RING_MIN_H: u16 = 10;

/// Fixed clock positions (x, y) of the 12 pitch classes on the ring canvas.
/// Index is the pitch class; `0` (C) sits at 12 o'clock, ascending clockwise.
const DODECA_NODES: [(usize, usize); 12] = [
    (11, 1),
    (16, 1),
    (19, 3),
    (20, 4),
    (19, 6),
    (16, 7),
    (11, 7),
    (7, 7),
    (3, 6),
    (2, 4),
    (3, 3),
    (7, 1),
];

/// Clock-degree label positions (x, y), placed just outside each node.
const DODECA_LABELS: [(usize, usize); 12] = [
    (11, 0),
    (18, 0),
    (21, 2),
    (22, 4),
    (21, 7),
    (18, 8),
    (11, 8),
    (5, 8),
    (1, 7),
    (0, 4),
    (1, 2),
    (5, 0),
];

/// Canvas dimensions of the full circular dodecagram.
const DODECA_CANVAS_W: usize = 23;
const DODECA_CANVAS_H: usize = 9;

/// Width required to draw the widest expanded pad row plus the block borders.
fn expanded_required_width() -> u16 {
    let max_keys = ALL_ROWS.iter().map(|row| row.len()).max().unwrap_or(0);
    let max_stagger = ROW_STAGGERS.iter().copied().max().unwrap_or(0);
    u16::try_from(max_keys * PAD_CELL_WIDTH + max_stagger + 2).unwrap_or(u16::MAX)
}

/// Whether the grid should render as multi-line box pads instead of compact text.
fn use_expanded_layout(area: Rect) -> bool {
    area.height >= EXPANDED_MIN_HEIGHT && area.width >= expanded_required_width()
}

/// Resolves the illumination state for a single pad, in precedence order.
fn pad_visual_state(app: &App, row: usize, col: usize, midi_note: Option<u8>) -> PadVisualState {
    let is_held = app.active_coords.contains(&(row, col));
    let is_sustained = midi_note.is_some_and(|n| app.sustained_notes.contains(&n));
    let is_root = midi_note.is_some_and(|n| n % 12 == app.root_note % 12);
    let is_accidental = midi_note.is_some_and(|n| pitch_class_name(n).contains('#'));

    if is_held {
        PadVisualState::ActiveHeld
    } else if is_sustained {
        PadVisualState::Sustained
    } else if is_root {
        PadVisualState::Root
    } else if is_accidental {
        PadVisualState::RestingAccidental
    } else {
        PadVisualState::RestingNatural
    }
}

/// Maps an illumination state to a terminal style drawn from the active palette.
fn pad_style(state: PadVisualState, palette: &ColorPalette) -> Style {
    match state {
        PadVisualState::ActiveHeld => Style::default()
            .fg(palette.held_fg)
            .bg(palette.held_bg)
            .add_modifier(Modifier::BOLD),
        PadVisualState::Sustained => Style::default()
            .fg(palette.held_fg)
            .bg(palette.sustain_bg)
            .add_modifier(Modifier::BOLD),
        PadVisualState::Root => Style::default()
            .fg(palette.root)
            .add_modifier(Modifier::BOLD),
        PadVisualState::RestingAccidental => {
            Style::default().fg(palette.accidental_fg).bg(palette.bg)
        }
        PadVisualState::RestingNatural => Style::default().fg(palette.natural_fg),
    }
}

/// Column split for the bottom panels: `(chord %, dodecagram %)`.
///
/// Below 100 columns the dodecagram is squeezed below `DODECA_RING_MIN_W`, so it
/// collapses into its compact clock; at 100+ it has room for the full ring.
fn dodecagram_column_split(frame_width: u16) -> (u16, u16) {
    if frame_width >= WIDE_LAYOUT_MIN_WIDTH {
        (58, 42)
    } else {
        (60, 40)
    }
}

/// Height of the combined chord + dodecagram panel, adaptive to terminal height.
fn bottom_panel_height(size: Rect) -> u16 {
    if size.height >= 30 {
        12
    } else if size.height >= 26 {
        10
    } else if size.height >= 22 {
        8
    } else {
        5
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                         // Header bar
            Constraint::Min(12),                           // Keyboard grid
            Constraint::Length(bottom_panel_height(size)), // Harmonic panels
            Constraint::Length(3),                         // Navigation / shortcuts footer
        ])
        .split(size);

    let (header_area, grid_area, bottom_area, footer_area) = match chunks.as_ref() {
        [h, g, b, f] => (*h, *g, *b, *f),
        _ => return,
    };

    let pc_set = active_pitch_class_set(app);

    render_header(frame, header_area, app);
    render_keyboard_grid(frame, grid_area, app);

    // Side-by-side on wide terminals; a narrower chord column otherwise so the
    // dodecagram collapses into its compact clock form below 100 columns.
    let (chord_pct, dodec_pct) = dodecagram_column_split(size.width);
    let columns = Layout::horizontal([
        Constraint::Percentage(chord_pct),
        Constraint::Percentage(dodec_pct),
    ])
    .split(bottom_area);

    match columns.as_ref() {
        [chord_area, dodec_area] => {
            render_chord_panel(frame, *chord_area, app);
            render_dodecagram_panel(frame, *dodec_area, app, pc_set);
        }
        _ => render_chord_panel(frame, bottom_area, app),
    }

    render_footer(frame, footer_area, app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let root_str = note_to_name(app.root_note);
    let sustain_color = if app.is_space_held {
        palette.sustain_bg
    } else {
        palette.accidental_fg
    };

    let synth_span = match app.synth.as_ref() {
        Some(s) if s.is_muted() => Span::styled("MUTED", Style::default().fg(palette.accent)),
        Some(s) => Span::styled(
            format!("Glicol ({})", s.preset_name()),
            Style::default()
                .fg(palette.lead)
                .add_modifier(Modifier::BOLD),
        ),
        None => Span::styled("OFF", Style::default().fg(palette.accidental_fg)),
    };

    let header_line = Line::from(vec![
        Span::styled(
            " ISOKBD ",
            Style::default()
                .fg(palette.lead)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ Theme: "),
        Span::styled(
            app.theme.name(),
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ Mode: "),
        Span::styled(
            app.layout_mode.name(),
            Style::default()
                .fg(palette.lead)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ Root: "),
        Span::styled(
            format!("{root_str} ({})", app.root_note),
            Style::default()
                .fg(palette.root)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ Vel: "),
        Span::styled(
            format!("{}", app.velocity),
            Style::default().fg(palette.natural_fg),
        ),
        Span::raw(" │ Pedal: "),
        Span::styled(
            if app.is_space_held { "DOWN" } else { "UP" },
            Style::default()
                .fg(sustain_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ Synth: "),
        synth_span,
        Span::raw(" │ Port: "),
        Span::styled(app.midi.port_name(), Style::default().fg(palette.accent)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accidental_fg));

    let paragraph = Paragraph::new(header_line)
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

pub fn render_keyboard_grid(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let block = Block::default()
        .title(" Isomorphic QWERTY Grid ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.lead));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let expanded = use_expanded_layout(area);
    let mut lines = Vec::new();

    // Render from row 3 (Number row) down to row 0 (Bottom letter row)
    for (row_idx, &keys) in ALL_ROWS.iter().enumerate().rev() {
        let stagger = ROW_STAGGERS.get(row_idx).copied().unwrap_or(0);

        let note_strings: Vec<String> = keys
            .iter()
            .map(|key| {
                calculate_midi_note(row_idx, key.col, app.root_note, app.layout_mode)
                    .map(note_to_name)
                    .unwrap_or_else(|| "--".to_string())
            })
            .collect();

        let cells: Vec<PadCell<'_>> = keys
            .iter()
            .zip(note_strings.iter())
            .map(|(key, note_str)| PadCell {
                key_char: key.label,
                note_name: note_str.as_str(),
                state: pad_visual_state(
                    app,
                    row_idx,
                    key.col,
                    calculate_midi_note(row_idx, key.col, app.root_note, app.layout_mode),
                ),
            })
            .collect();

        if expanded {
            lines.extend(render_expanded_row(stagger, &cells, &palette));
        } else {
            lines.push(render_compact_row(stagger, &cells, &palette));
            if row_idx > 0 {
                lines.push(Line::raw(""));
            }
        }
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

/// Builds the three box-drawn lines (`┌───┐`, `│...│`, `└───┘`) for one key row.
fn render_expanded_row(
    stagger: usize,
    cells: &[PadCell<'_>],
    palette: &ColorPalette,
) -> Vec<Line<'static>> {
    let mut top: Vec<Span<'static>> = Vec::new();
    let mut mid: Vec<Span<'static>> = Vec::new();
    let mut bottom: Vec<Span<'static>> = Vec::new();

    if stagger > 0 {
        let pad = Span::raw(" ".repeat(stagger));
        top.push(pad.clone());
        mid.push(pad.clone());
        bottom.push(pad);
    }

    for cell in cells {
        let style = pad_style(cell.state, palette);
        let content = format!("{} {:<3}", cell.key_char, cell.note_name);
        top.push(Span::styled("┌─────┐".to_string(), style));
        mid.push(Span::styled(format!("│{content}│"), style));
        bottom.push(Span::styled("└─────┘".to_string(), style));
    }

    vec![Line::from(top), Line::from(mid), Line::from(bottom)]
}

/// Collects the currently sounding notes into a 12-bit pitch-class set.
fn active_pitch_class_set(app: &App) -> PitchClassSet {
    let notes: Vec<u8> = app.active_notes.iter().copied().collect();
    PitchClassSet::from_midi_notes(&notes)
}

/// Writes `ch` into the canvas, ignoring out-of-bounds coordinates.
fn put_char(canvas: &mut [Vec<char>], x: usize, y: usize, ch: char) {
    if let Some(row) = canvas.get_mut(y)
        && let Some(cell) = row.get_mut(x)
    {
        *cell = ch;
    }
}

/// Chooses a line glyph for a segment with the given deltas (screen coords).
fn line_glyph(dx: i32, dy: i32) -> char {
    if dy == 0 {
        '─'
    } else if dx == 0 {
        '│'
    } else if (dx > 0) == (dy > 0) {
        '╲'
    } else {
        '╱'
    }
}

/// Draws a straight connecting line between two canvas points.
fn draw_line(canvas: &mut [Vec<char>], from: (usize, usize), to: (usize, usize)) {
    let (x0, y0) = (from.0 as i32, from.1 as i32);
    let (x1, y1) = (to.0 as i32, to.1 as i32);
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = dx.abs().max(dy.abs());
    if steps == 0 {
        return;
    }

    let glyph = line_glyph(dx, dy);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let x = (x0 as f32 + dx as f32 * t).round() as i32;
        let y = (y0 as f32 + dy as f32 * t).round() as i32;
        if x < 0 || y < 0 {
            continue;
        }
        if let Some(row) = canvas.get_mut(y as usize)
            && let Some(cell) = row.get_mut(x as usize)
            && *cell == ' '
        {
            // Lines never clobber nodes or labels already placed.
            *cell = glyph;
        }
    }
}

/// Builds the 23x9 character canvas of the circular dodecagram.
fn build_dodecagram_canvas(pc_set: PitchClassSet) -> Vec<Vec<char>> {
    let mut canvas: Vec<Vec<char>> = vec![vec![' '; DODECA_CANVAS_W]; DODECA_CANVAS_H];

    // 1. Connect consecutive active pitch classes into a closed polygon.
    let active: Vec<usize> = (0..12usize)
        .filter(|&pc| pc_set.contains(pc as u8))
        .collect();
    if active.len() >= 2 {
        for window in active.windows(2) {
            if let (Some(&from), Some(&to)) = (window.first(), window.get(1))
                && let (Some(&a), Some(&b)) = (DODECA_NODES.get(from), DODECA_NODES.get(to))
            {
                draw_line(&mut canvas, a, b);
            }
        }
        if let (Some(&last), Some(&first)) = (active.last(), active.first())
            && let (Some(&a), Some(&b)) = (DODECA_NODES.get(last), DODECA_NODES.get(first))
        {
            draw_line(&mut canvas, a, b);
        }
    }

    // 2. Nodes: illuminated filled for active degrees, muted open otherwise.
    for pc in 0..12usize {
        let glyph = if pc_set.contains(pc as u8) {
            '●'
        } else {
            '○'
        };
        if let Some(&(x, y)) = DODECA_NODES.get(pc) {
            put_char(&mut canvas, x, y, glyph);
        }
    }

    // 3. Clock-degree labels (12, 1, ..., 11).
    for pc in 0..12usize {
        let label = if pc == 0 {
            "12".to_string()
        } else {
            pc.to_string()
        };
        if let Some(&(lx, ly)) = DODECA_LABELS.get(pc) {
            for (offset, ch) in label.chars().enumerate() {
                put_char(&mut canvas, lx + offset, ly, ch);
            }
        }
    }

    canvas
}

/// Coarse glyph category, used to batch a canvas row into as few spans as possible.
fn glyph_kind(ch: char) -> u8 {
    match ch {
        '●' => 0,
        '○' => 1,
        '─' | '│' | '╱' | '╲' => 2,
        _ => 3,
    }
}

fn glyph_style(kind: u8, palette: &ColorPalette) -> Style {
    match kind {
        0 => Style::default()
            .fg(palette.lead)
            .add_modifier(Modifier::BOLD),
        1 => Style::default().fg(palette.accidental_fg),
        2 => Style::default().fg(palette.accent),
        _ => Style::default().fg(palette.natural_fg),
    }
}

/// Converts one canvas row into a styled line, grouping same-style runs.
fn canvas_line(row: &[char], palette: &ColorPalette) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_kind: Option<u8> = None;

    for &ch in row {
        let kind = glyph_kind(ch);
        if run_kind == Some(kind) {
            run.push(ch);
        } else {
            if let Some(previous) = run_kind
                && !run.is_empty()
            {
                spans.push(Span::styled(
                    std::mem::take(&mut run),
                    glyph_style(previous, palette),
                ));
            }
            run_kind = Some(kind);
            run.push(ch);
        }
    }

    if let Some(previous) = run_kind
        && !run.is_empty()
    {
        spans.push(Span::styled(run, glyph_style(previous, palette)));
    }

    Line::from(spans)
}

/// A single labelled compact-clock cell, e.g. `12●` or ` 7○`.
fn compact_cell(pc_set: PitchClassSet, pc: usize) -> String {
    let label = if pc == 0 {
        "12".to_string()
    } else {
        pc.to_string()
    };
    let glyph = if pc_set.contains(pc as u8) {
        '●'
    } else {
        '○'
    };
    format!("{label:>2}{glyph}")
}

/// The compact clock fallback, linearising the ring when the panel is narrow.
fn compact_dodecagram_lines(
    pc_set: PitchClassSet,
    width: u16,
    palette: &ColorPalette,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let style = Style::default().fg(palette.natural_fg);

    if width >= 28 {
        let row_a: Vec<String> = (0..6).map(|pc| compact_cell(pc_set, pc)).collect();
        let row_b: Vec<String> = (6..12).map(|pc| compact_cell(pc_set, pc)).collect();
        lines.push(Line::from(Span::styled(row_a.join("  "), style)));
        lines.push(Line::from(Span::styled(row_b.join("  "), style)));
    } else if width >= 12 {
        let row: Vec<&str> = (0..12)
            .map(|pc| {
                if pc_set.contains(pc as u8) {
                    "●"
                } else {
                    "○"
                }
            })
            .collect();
        lines.push(Line::from(Span::styled(row.join(""), style)));
    }

    lines
}

/// Renders the complement of a pitch-class set as text, truncated with an
/// ellipsis so it always fits within `width` columns.
fn complement_summary_fitted(complement: PitchClassSet, width: usize) -> String {
    let names = complement.pitch_class_names();
    let prefix = format!("Complement ({}): {{", complement.count());
    if names.is_empty() {
        return format!("{prefix}}}");
    }

    let mut body = String::new();
    for (index, name) in names.iter().enumerate() {
        let candidate = if index == 0 {
            (*name).to_string()
        } else {
            format!("{body}, {name}")
        };
        // Reserve room for a closing `}` (or `, …}` when truncating).
        let is_last = index + 1 == names.len();
        let tail = if is_last { 1 } else { 4 };
        if prefix.chars().count() + candidate.chars().count() + tail > width {
            return if body.is_empty() {
                format!("Complement ({}):", complement.count())
            } else {
                format!("{prefix}{body}, …}}")
            };
        }
        body = candidate;
    }

    format!("{prefix}{body}}}")
}

pub fn render_dodecagram_panel(frame: &mut Frame, area: Rect, app: &App, pc_set: PitchClassSet) {
    let palette = app.theme.palette();
    let block = Block::default()
        .title(Line::from(format!(
            " Dodecagram · Root {} ",
            note_to_name(app.root_note)
        )))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accidental_fg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let complement_line = complement_summary_fitted(pc_set.complement(), usize::from(inner.width));

    let mut lines: Vec<Line<'static>> = Vec::new();
    if inner.width >= DODECA_RING_MIN_W && inner.height >= DODECA_RING_MIN_H {
        for row in build_dodecagram_canvas(pc_set) {
            lines.push(canvas_line(&row, &palette));
        }
    } else {
        lines.extend(compact_dodecagram_lines(pc_set, inner.width, &palette));
    }

    lines.push(Line::from(Span::styled(
        complement_line,
        Style::default()
            .fg(palette.accent)
            .add_modifier(Modifier::BOLD),
    )));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

/// Builds the compact single-line rendering, e.g. `[1:D#4]`.
fn render_compact_row(
    stagger: usize,
    cells: &[PadCell<'_>],
    palette: &ColorPalette,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();

    if stagger > 0 {
        spans.push(Span::raw(" ".repeat(stagger)));
    }

    for cell in cells {
        let style = pad_style(cell.state, palette);
        let content = format!(" [{:>1}:{:<3}]", cell.key_char, cell.note_name);
        spans.push(Span::styled(content, style));
    }

    Line::from(spans)
}

fn render_chord_panel(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let chord_title = app.detected_chord.as_deref().unwrap_or("No notes playing");
    let pc_set = active_pitch_class_set(app);
    let active_notes_str: Vec<String> = app.active_notes.iter().map(|&n| note_to_name(n)).collect();
    let notes_summary = if active_notes_str.is_empty() {
        "—".to_string()
    } else {
        active_notes_str.join(", ")
    };

    let formula = pc_set.interval_formula();
    let formula_str = if formula.is_empty() {
        "[]".to_string()
    } else {
        format!("{formula:?}")
    };
    let density_str = format!("{}/12", pc_set.count());
    let barcode = pc_set.barcode_string();

    let label_style = Style::default().fg(palette.accidental_fg);
    let content = vec![
        Line::from(vec![
            Span::styled("Chord: ", label_style),
            Span::styled(
                chord_title,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            if let Some(ref msg) = app.status_message {
                Span::styled(format!("  •  {msg}"), Style::default().fg(palette.lead))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(vec![
            Span::styled("Formula: ", label_style),
            Span::styled(formula_str, Style::default().fg(palette.natural_fg)),
            Span::styled("  •  Density: ", label_style),
            Span::styled(
                density_str,
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Barcode: ", label_style),
            Span::styled(barcode, Style::default().fg(palette.lead)),
        ]),
        Line::from(vec![
            Span::styled("Active:  ", label_style),
            Span::styled(notes_summary, Style::default().fg(palette.natural_fg)),
            if let Some(ref s) = app.synth {
                Span::styled(format!("  •  Audio Out: {}", s.device_name()), label_style)
            } else {
                Span::raw("")
            },
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(label_style);

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let palette = app.theme.palette();
    let key_style = Style::default()
        .fg(palette.lead)
        .add_modifier(Modifier::BOLD);
    let footer_line = Line::from(vec![
        Span::styled(" [↑/↓] ", key_style),
        Span::raw("Octave  "),
        Span::styled(" [←/→] ", key_style),
        Span::raw("Transpose  "),
        Span::styled(" [Tab] ", key_style),
        Span::raw("Layout  "),
        Span::styled(" [Space] ", key_style),
        Span::raw("Pedal (Hold)  "),
        Span::styled(" [Enter] ", key_style),
        Span::raw("Synth  "),
        Span::styled(" [PgUp/PgDn] ", key_style),
        Span::raw("Vel  "),
        Span::styled(" [T] ", key_style),
        Span::raw("Theme  "),
        Span::styled(
            " [Esc/Ctrl+C] ",
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Quit"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accidental_fg));

    let paragraph = Paragraph::new(footer_line)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::app::Theme;
    use crate::midi::MidiEngine;

    fn test_app() -> App {
        App::new(MidiEngine::silent(), None)
    }

    #[test]
    fn pad_state_selection_and_precedence() {
        let mut app = test_app();

        // Wicki-Hayden with base 41 (F2): 'S' (1,1) resolves to C3 = 48.
        let note_c3 = calculate_midi_note(1, 1, app.root_note, app.layout_mode);
        assert_eq!(note_c3, Some(48));
        assert_eq!(
            pad_visual_state(&app, 1, 1, note_c3),
            PadVisualState::RestingNatural
        );

        // (1,4) resolves to F#3 = 54: an accidental at rest.
        let note_fs3 = calculate_midi_note(1, 4, app.root_note, app.layout_mode);
        assert_eq!(note_fs3, Some(54));
        assert_eq!(
            pad_visual_state(&app, 1, 4, note_fs3),
            PadVisualState::RestingAccidental
        );

        // (0,0) resolves to F2 = 41, which is the current root.
        let note_f2 = calculate_midi_note(0, 0, app.root_note, app.layout_mode);
        assert_eq!(note_f2, Some(41));
        assert_eq!(pad_visual_state(&app, 0, 0, note_f2), PadVisualState::Root);

        // Root highlight outranks the accidental surface even for a sharp root.
        app.root_note = 54;
        assert!(pitch_class_name(54).contains('#'));
        assert_eq!(pad_visual_state(&app, 1, 4, note_fs3), PadVisualState::Root);

        // A physically held key outranks sustained and root.
        app.active_coords.insert((1, 4));
        app.sustained_notes.insert(54);
        assert_eq!(
            pad_visual_state(&app, 1, 4, note_fs3),
            PadVisualState::ActiveHeld
        );

        // Released but still ringing under the sustain pedal.
        app.active_coords.remove(&(1, 4));
        assert_eq!(
            pad_visual_state(&app, 1, 4, note_fs3),
            PadVisualState::Sustained
        );

        // A pad with no mapped note is a resting natural.
        assert_eq!(
            pad_visual_state(&app, 1, 4, None),
            PadVisualState::RestingNatural
        );
    }

    #[test]
    fn compact_vs_expanded_threshold() {
        // Tall enough and wide enough -> box pads.
        assert!(use_expanded_layout(Rect::new(0, 0, 120, 20)));
        // Tall but too narrow to fit the widest pad row -> compact.
        assert!(!use_expanded_layout(Rect::new(0, 0, 60, 20)));
        // Wide but vertically constrained -> compact.
        assert!(!use_expanded_layout(Rect::new(0, 0, 120, 10)));
        // Exactly at the documented height threshold.
        assert!(use_expanded_layout(Rect::new(
            0,
            0,
            120,
            EXPANDED_MIN_HEIGHT
        )));
        assert_eq!(expanded_required_width(), 92);
    }

    #[test]
    fn row_stagger_offsets_preserved() {
        assert_eq!(ROW_STAGGERS, [6, 4, 2, 0]);
    }

    #[test]
    fn dodecagram_canvas_marks_nodes() {
        let set = PitchClassSet::from_midi_notes(&[48, 52, 55]); // C, E, G
        let canvas = build_dodecagram_canvas(set);
        for (pc, &(x, y)) in DODECA_NODES.iter().enumerate() {
            let glyph = canvas[y][x];
            if set.contains(pc as u8) {
                assert_eq!(glyph, '●', "pc {pc} should be an illuminated node");
            } else {
                assert_eq!(glyph, '○', "pc {pc} should be a muted node");
            }
        }
    }

    #[test]
    fn dodecagram_connects_active_nodes_with_lines() {
        let triad = build_dodecagram_canvas(PitchClassSet::from_midi_notes(&[48, 52, 55]));
        let triad_lines = triad
            .iter()
            .flatten()
            .filter(|&&c| matches!(c, '─' | '│' | '╱' | '╲'))
            .count();
        assert!(triad_lines > 0, "a triad should draw a connecting triangle");

        // A lone note has no polygon to connect.
        let single = build_dodecagram_canvas(PitchClassSet::from_midi_notes(&[48]));
        let single_lines = single
            .iter()
            .flatten()
            .filter(|&&c| matches!(c, '─' | '│' | '╱' | '╲'))
            .count();
        assert_eq!(single_lines, 0);
    }

    #[test]
    fn dodecagram_complement_summary_text() {
        let set = PitchClassSet::from_midi_notes(&[48, 52, 55, 59]); // C E G B
        let summary = complement_summary_fitted(set.complement(), 100);
        assert_eq!(summary, "Complement (8): {C#, D, D#, F, F#, G#, A, A#}");

        // An empty set's complement is the full chromatic field.
        let full = complement_summary_fitted(PitchClassSet::default().complement(), 100);
        assert_eq!(
            full,
            "Complement (12): {C, C#, D, D#, E, F, F#, G, G#, A, A#, B}"
        );

        // Narrow panels truncate cleanly with an ellipsis instead of clipping.
        let narrow = complement_summary_fitted(set.complement(), 30);
        assert!(narrow.ends_with("…}"), "got: {narrow}");
        assert!(narrow.chars().count() <= 30, "got: {narrow}");
        // Degenerate widths fall back to just the count.
        assert_eq!(
            complement_summary_fitted(set.complement(), 4),
            "Complement (8):"
        );
    }

    #[test]
    fn compact_dodecagram_collapses_by_width() {
        let set = PitchClassSet::from_midi_notes(&[48, 52, 55]);
        let palette = Theme::ClassicAnsi.palette();
        assert_eq!(compact_dodecagram_lines(set, 40, &palette).len(), 2);
        assert_eq!(compact_dodecagram_lines(set, 20, &palette).len(), 1);
        assert_eq!(compact_dodecagram_lines(set, 8, &palette).len(), 0);
    }

    #[test]
    fn render_smoke_across_terminal_sizes() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let mut app = test_app();
        // A C Maj7 voicing: C3, E3, G3, B3, with 'S' physically held.
        for note in [48u8, 52, 55, 59] {
            app.active_notes.insert(note);
        }
        app.active_coords.insert((1, 1));
        app.detected_chord = Some("C Maj7".to_string());

        // Every size must render without panicking; the documented expanded
        // (120x35) and compact (80x24) targets must also show all three widgets.
        let documented = [(120u16, 35u16), (80, 24)];
        for (width, height) in documented
            .into_iter()
            .chain([(60u16, 20u16), (200, 50), (40, 12)])
        {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).expect("test backend");
            let draw = terminal.draw(|frame| render(frame, &app));
            assert!(draw.is_ok(), "render panicked/failed at {width}x{height}");

            let text: String = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect();
            if documented.contains(&(width, height)) {
                assert!(
                    text.contains("Genese LUFS"),
                    "theme missing at {width}x{height}"
                );
                assert!(
                    text.contains("Barcode"),
                    "barcode missing at {width}x{height}"
                );
                assert!(
                    text.contains("Complement"),
                    "complement missing at {width}x{height}"
                );
            }
        }
    }

    #[test]
    fn grid_expands_to_box_pads_only_when_space_allows() {
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        let app = test_app();
        let render_text = |width: u16, height: u16| -> String {
            let mut terminal =
                Terminal::new(TestBackend::new(width, height)).expect("test backend");
            terminal.draw(|frame| render(frame, &app)).expect("draw");
            terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect()
        };

        // Expanded target: box-drawn pads.
        assert!(render_text(120, 35).contains('┌'));
        // Compact target: no box glyphs, single-line pads only.
        assert!(!render_text(80, 24).contains('┌'));
    }

    #[test]
    fn pad_style_tracks_active_palette() {
        // The held-key background follows the palette's `held_bg` token.
        let lufs = Theme::GeneseLufs.palette();
        let ansi = Theme::ClassicAnsi.palette();
        assert_eq!(
            pad_style(PadVisualState::ActiveHeld, &lufs).bg,
            Some(lufs.held_bg)
        );
        assert_eq!(
            pad_style(PadVisualState::ActiveHeld, &ansi).bg,
            Some(ansi.held_bg)
        );
        assert_ne!(lufs.held_bg, ansi.held_bg);
    }

    #[test]
    fn dodecagram_collapses_below_100_columns() {
        let dodec_width = |frame: u16| {
            let (_, pct) = dodecagram_column_split(frame);
            frame * pct / 100
        };
        // At 100+ columns the panel is wide enough for the circular ring...
        assert!(dodec_width(120) >= DODECA_RING_MIN_W);
        assert!(dodec_width(100) >= DODECA_RING_MIN_W);
        // ...but below 100 it falls under the ring threshold and collapses.
        assert!(dodec_width(99) < DODECA_RING_MIN_W);
        assert!(dodec_width(80) < DODECA_RING_MIN_W);
    }

    #[test]
    fn bottom_panel_height_scales_with_terminal() {
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 120, 40)), 12);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 120, 26)), 10);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 80, 24)), 8);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 80, 20)), 5);
    }
}
