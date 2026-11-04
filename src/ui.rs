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
/// The ring canvas is 23 columns wide, so 25 leaves a column of breathing room
/// on each side; the height covers the 9 canvas rows plus the complement line.
const DODECA_RING_MIN_W: u16 = 25;
const DODECA_RING_MIN_H: u16 = 10;

/// Glyph used for the dodecagram's perimeter ring.
///
/// The perimeter is drawn as a dotted circle so the figure always reads as a
/// twelve-point dodecagram, even when a single note means there is no chord
/// polygon to draw. It stays visually subordinate to the solid chord lines.
const DODECA_RING_CHAR: char = '·'; // ·

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

/// Cell density for the single-line compact grid, chosen from the columns
/// available in the grid's inner area so a row never runs off the right edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompactDensity {
    /// ` [1:G#3]` — 8 columns per pad.
    Full,
    /// ` 1:G#3` — 6 columns per pad.
    Tight,
    /// ` 1` — 2 columns per pad; note names are dropped, key labels remain.
    KeyOnly,
}

impl CompactDensity {
    /// Picks the widest density whose stride still fits `available` columns.
    fn for_row(available: usize, keys: usize) -> Self {
        let stride = available.checked_div(keys).unwrap_or(0);
        if stride >= 8 {
            Self::Full
        } else if stride >= 6 {
            Self::Tight
        } else {
            Self::KeyOnly
        }
    }
}

/// Fixed width for the dodecagram panel: its 23-column canvas, two block
/// borders, and a little breathing room on either side.
const DODECA_PANEL_WIDTH: u16 = 32;

/// Horizontal split for the bottom panels.
///
/// On wide terminals the dodecagram takes a fixed width sized to its canvas and
/// the chord panel absorbs the remainder, which keeps the ring's proportions
/// stable as the window grows instead of stretching into empty space.
fn dodecagram_columns(frame_width: u16) -> [Constraint; 2] {
    if frame_width >= WIDE_LAYOUT_MIN_WIDTH {
        [Constraint::Min(40), Constraint::Length(DODECA_PANEL_WIDTH)]
    } else {
        [Constraint::Percentage(60), Constraint::Percentage(40)]
    }
}

/// Prepends blank lines so a block of rendered lines sits vertically centred
/// inside `inner`.
fn center_vertically(lines: Vec<Line<'static>>, inner: Rect) -> Vec<Line<'static>> {
    let content_height = lines.len();
    let top_pad = usize::from(
        inner
            .height
            .saturating_sub(u16::try_from(content_height).unwrap_or(u16::MAX)),
    ) / 2;

    let mut centred: Vec<Line<'static>> = Vec::with_capacity(top_pad + content_height);
    for _ in 0..top_pad {
        centred.push(Line::raw(""));
    }
    centred.extend(lines);
    centred
}

/// Centres a block of rendered lines both horizontally and vertically inside
/// `inner`, so the instrument sits in the middle of its panel rather than
/// hugging the top-left corner on large terminals.
fn center_block(lines: Vec<Line<'static>>, inner: Rect) -> Vec<Line<'static>> {
    let content_width = lines.iter().map(Line::width).max().unwrap_or(0);
    let content_height = lines.len();

    let to_u16 = |value: usize| u16::try_from(value).unwrap_or(u16::MAX);
    let left_pad = usize::from(inner.width.saturating_sub(to_u16(content_width))) / 2;
    let top_pad = usize::from(inner.height.saturating_sub(to_u16(content_height))) / 2;

    let mut centred: Vec<Line<'static>> = Vec::with_capacity(top_pad + content_height);
    for _ in 0..top_pad {
        centred.push(Line::raw(""));
    }
    for line in lines {
        if left_pad == 0 {
            centred.push(line);
        } else {
            let mut spans = line.spans;
            spans.insert(0, Span::raw(" ".repeat(left_pad)));
            centred.push(Line::from(spans));
        }
    }
    centred
}

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

/// Rows claimed by the header bar and the shortcuts footer together.
const CHROME_HEIGHT: u16 = 6;

/// Smallest grid the layout will hand the keyboard before squeezing it:
/// four compact key rows plus the two block borders.
const GRID_MIN_HEIGHT: u16 = 6;

/// Height of the combined chord + dodecagram panel, adaptive to terminal height.
///
/// The preferred height is clamped to whatever is left once the header, footer
/// and a usable keyboard grid have taken their rows. Without the clamp the
/// vertical constraints over-subscribe short terminals (60x20 asks for
/// 3 + 12 + 5 + 3 = 23 rows) and Ratatui collapses the footer down to its two
/// border rows, silently deleting the shortcut hints.
fn bottom_panel_height(size: Rect) -> u16 {
    let preferred = if size.height >= 30 {
        12
    } else if size.height >= 26 {
        10
    } else if size.height >= 22 {
        8
    } else {
        5
    };
    let spare = size
        .height
        .saturating_sub(CHROME_HEIGHT)
        .saturating_sub(GRID_MIN_HEIGHT);
    // Below three rows a harmonic panel is just two borders around no content,
    // so drop it entirely and give the rows back to the playable surface.
    if spare < 3 { 0 } else { preferred.min(spare) }
}

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                         // Header bar
            Constraint::Min(GRID_MIN_HEIGHT),              // Keyboard grid
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
    // A zero-height panel means the terminal is too short to spare the rows.
    if bottom_area.height >= 3 {
        let columns = Layout::horizontal(dodecagram_columns(size.width)).split(bottom_area);

        match columns.as_ref() {
            [chord_area, dodec_area] => {
                render_chord_panel(frame, *chord_area, app);
                render_dodecagram_panel(frame, *dodec_area, app, pc_set);
            }
            _ => render_chord_panel(frame, bottom_area, app),
        }
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

    let bold =
        |color: ratatui::style::Color| Style::default().fg(color).add_modifier(Modifier::BOLD);

    let (synth_value, synth_style) = match app.synth.as_ref() {
        Some(s) if s.is_muted() => ("MUTED".to_string(), bold(palette.accent)),
        Some(s) => (format!("Glicol ({})", s.preset_name()), bold(palette.lead)),
        None => (
            "OFF".to_string(),
            Style::default().fg(palette.accidental_fg),
        ),
    };

    // Fields in priority order. When the terminal is too narrow the tail is
    // dropped a whole field at a time, so the bar never slices a value in half.
    let fields = [
        (
            "Theme: ",
            app.theme.name().to_string(),
            bold(palette.accent),
        ),
        (
            "Mode: ",
            app.layout_mode.name().to_string(),
            bold(palette.lead),
        ),
        (
            "Root: ",
            format!("{root_str} ({})", app.root_note),
            bold(palette.root),
        ),
        (
            "Vel: ",
            app.velocity.to_string(),
            Style::default().fg(palette.natural_fg),
        ),
        (
            "Pedal: ",
            if app.is_space_held { "DOWN" } else { "UP" }.to_string(),
            bold(sustain_color),
        ),
        ("Synth: ", synth_value, synth_style),
        (
            "Port: ",
            app.midi.port_name(),
            Style::default().fg(palette.accent),
        ),
    ];

    const BRAND: &str = " ISOKBD ";
    const SEPARATOR: &str = "\u{2502}"; // │

    let separator_style = Style::default().fg(palette.accidental_fg);
    let available = usize::from(area.width.saturating_sub(2));
    let mut spans = vec![Span::styled(BRAND, bold(palette.lead))];
    let mut used = BRAND.chars().count();

    for (label, value, style) in fields {
        let cost = SEPARATOR.chars().count() + label.chars().count() + value.chars().count();
        if used + cost > available {
            break;
        }
        used += cost;
        spans.push(Span::styled(SEPARATOR, separator_style));
        spans.push(Span::raw(label));
        spans.push(Span::styled(value, style));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accidental_fg));

    let paragraph = Paragraph::new(Line::from(spans))
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
            lines.extend(render_expanded_row(
                stagger,
                &cells,
                &palette,
                app.show_labels,
            ));
        } else {
            let available = usize::from(inner.width).saturating_sub(stagger);
            let density = CompactDensity::for_row(available, cells.len());
            lines.push(render_compact_row(
                stagger,
                &cells,
                &palette,
                density,
                app.show_labels,
            ));
            if row_idx > 0 {
                lines.push(Line::raw(""));
            }
        }
    }

    // Centre the keyboard within its panel: the pad block is a fixed size, so
    // on large terminals it otherwise hugs the top-left corner.
    let centred = center_block(lines, inner);
    let paragraph = Paragraph::new(centred).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

/// Builds the three box-drawn lines (`┌───┐`, `│...│`, `└───┘`) for one key row.
fn render_expanded_row(
    stagger: usize,
    cells: &[PadCell<'_>],
    palette: &ColorPalette,
    show_labels: bool,
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
        // With labels hidden the pad keeps its tactile border and its
        // border colour still carries the illumination state, but the interior
        // is left empty so the grid reads as a clean field of blank boxes.
        let content = if show_labels {
            format!("{} {:<3}", cell.key_char, cell.note_name)
        } else {
            "     ".to_string()
        };
        top.push(Span::styled("┌─────┐".to_string(), style));
        mid.push(Span::styled(format!("│{content}│"), style));
        bottom.push(Span::styled("└─────┘".to_string(), style));
    }

    vec![Line::from(top), Line::from(mid), Line::from(bottom)]
}

/// Single glyph summarising a pad's illumination state when labels are hidden.
fn pad_glyph(state: PadVisualState) -> char {
    match state {
        PadVisualState::ActiveHeld => '█',
        PadVisualState::Sustained => '▓',
        PadVisualState::Root => '◆',
        PadVisualState::RestingAccidental => '▫',
        PadVisualState::RestingNatural => '▪',
    }
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
fn line_glyph(dx: i64, dy: i64) -> char {
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
///
/// Uses integer Bresenham so each step advances the dominant axis by exactly
/// one cell. A float walker rounds two steps into the same row on shallow
/// segments and paints doubled glyphs (`╲╲`, `╱╱╱╱╱`), which reads as a broken
/// line in a terminal where cells are roughly twice as tall as they are wide.
fn draw_line(canvas: &mut [Vec<char>], from: (usize, usize), to: (usize, usize)) {
    let x0 = i64::try_from(from.0).unwrap_or(i64::MAX);
    let y0 = i64::try_from(from.1).unwrap_or(i64::MAX);
    let x1 = i64::try_from(to.0).unwrap_or(i64::MAX);
    let y1 = i64::try_from(to.1).unwrap_or(i64::MAX);

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    if dx == 0 && dy == 0 {
        return;
    }

    let glyph = line_glyph(x1 - x0, y1 - y0);
    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };
    // A cell is roughly twice as tall as it is wide, so the glyph for each
    // rasterized step follows the *cell* move: pure horizontal runs use `─`,
    // pure vertical runs use `│`, and only true corner-to-corner steps use a
    // diagonal. Stamping one diagonal glyph across a whole shallow segment
    // paints `╲╲` / `╱╱╱╱╱`, which reads as a broken line.
    let diagonal = if (step_x > 0) == (step_y > 0) {
        '╲'
    } else {
        '╱'
    };

    let mut error = dx - dy;
    let (mut x, mut y) = (x0, y0);
    let mut glyph = glyph;

    loop {
        if x >= 0
            && y >= 0
            && let Some(row) = canvas.get_mut(usize::try_from(y).unwrap_or(usize::MAX))
            && let Some(cell) = row.get_mut(usize::try_from(x).unwrap_or(usize::MAX))
            && (*cell == ' ' || *cell == DODECA_RING_CHAR)
        {
            // Lines never clobber nodes or labels already placed, but they do
            // overwrite the dotted perimeter so the chord polygon stays solid.
            *cell = glyph;
        }

        if x == x1 && y == y1 {
            break;
        }

        let doubled = 2 * error;
        let mut moved_x = false;
        let mut moved_y = false;
        if doubled > -dy {
            error -= dy;
            x += step_x;
            moved_x = true;
        }
        if doubled < dx {
            error += dx;
            y += step_y;
            moved_y = true;
        }
        glyph = match (moved_x, moved_y) {
            (true, false) => '─',
            (false, true) => '│',
            _ => diagonal,
        };
    }
}

/// Draws one dotted edge of the dodecagram's perimeter ring.
///
/// Mirrors [`draw_line`]'s integer walk, but stamps [`DODECA_RING_CHAR`] and
/// only fills empty cells so the solid chord polygon always wins.
fn draw_ring(canvas: &mut [Vec<char>], from: (usize, usize), to: (usize, usize)) {
    let x0 = i64::try_from(from.0).unwrap_or(i64::MAX);
    let y0 = i64::try_from(from.1).unwrap_or(i64::MAX);
    let x1 = i64::try_from(to.0).unwrap_or(i64::MAX);
    let y1 = i64::try_from(to.1).unwrap_or(i64::MAX);

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    if dx == 0 && dy == 0 {
        return;
    }

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut error = dx - dy;
    let (mut x, mut y) = (x0, y0);

    loop {
        if x >= 0
            && y >= 0
            && let Some(row) = canvas.get_mut(usize::try_from(y).unwrap_or(usize::MAX))
            && let Some(cell) = row.get_mut(usize::try_from(x).unwrap_or(usize::MAX))
            && *cell == ' '
        {
            *cell = DODECA_RING_CHAR;
        }

        if x == x1 && y == y1 {
            break;
        }

        let doubled = 2 * error;
        if doubled > -dy {
            error -= dy;
            x += step_x;
        }
        if doubled < dx {
            error += dx;
            y += step_y;
        }
    }
}

/// Whether the panel's inner area is large enough to draw the full ring.
///
/// Below either threshold the widget falls back to the compact clock, which
/// linearises the twelve degrees into labelled `12●` / ` 7○` cells.
fn dodecagram_shows_ring(inner: Rect) -> bool {
    inner.width >= DODECA_RING_MIN_W && inner.height >= DODECA_RING_MIN_H
}

/// Builds the 23x9 character canvas of the circular dodecagram.
fn build_dodecagram_canvas(pc_set: PitchClassSet) -> Vec<Vec<char>> {
    let mut canvas: Vec<Vec<char>> = vec![vec![' '; DODECA_CANVAS_W]; DODECA_CANVAS_H];

    // 0. Perimeter: a dotted ring through all twelve chromatic positions.
    for pc in 0..12usize {
        if let (Some(&a), Some(&b)) = (DODECA_NODES.get(pc), DODECA_NODES.get((pc + 1) % 12)) {
            draw_ring(&mut canvas, a, b);
        }
    }

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
        DODECA_RING_CHAR => 3,
        _ => 4,
    }
}

fn glyph_style(kind: u8, palette: &ColorPalette) -> Style {
    match kind {
        0 => Style::default()
            .fg(palette.lead)
            .add_modifier(Modifier::BOLD),
        1 => Style::default().fg(palette.accidental_fg),
        2 => Style::default().fg(palette.accent),
        // The perimeter stays dim so the chord polygon reads as the subject.
        3 => Style::default().fg(palette.accidental_fg),
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
    if dodecagram_shows_ring(inner) {
        // Centre the ring against the *panel* width, not against the widest
        // line in the block: the complement summary spans the full width and
        // would otherwise drag the figure to the left edge.
        let canvas_width = u16::try_from(DODECA_CANVAS_W).unwrap_or(u16::MAX);
        let left_pad = usize::from(inner.width.saturating_sub(canvas_width)) / 2;
        for row in build_dodecagram_canvas(pc_set) {
            let mut line = canvas_line(&row, &palette);
            if left_pad > 0 {
                line.spans.insert(0, Span::raw(" ".repeat(left_pad)));
            }
            lines.push(line);
        }
    } else {
        lines.extend(compact_dodecagram_lines(pc_set, inner.width, &palette));
    }

    lines.push(
        Line::from(Span::styled(
            complement_line,
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .centered(),
    );

    // Centre the figure vertically in the panel; horizontal centring is done
    // per-line above so the ring stays true to the panel's centre.
    let centred = center_vertically(lines, inner);
    let paragraph = Paragraph::new(centred).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

/// Builds the compact single-line rendering, e.g. `[1:D#4]`.
fn render_compact_row(
    stagger: usize,
    cells: &[PadCell<'_>],
    palette: &ColorPalette,
    density: CompactDensity,
    show_labels: bool,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();

    if stagger > 0 {
        spans.push(Span::raw(" ".repeat(stagger)));
    }

    for cell in cells {
        let style = pad_style(cell.state, palette);
        let content = if show_labels {
            match density {
                CompactDensity::Full => format!(" [{:>1}:{:<3}]", cell.key_char, cell.note_name),
                CompactDensity::Tight => format!(" {:>1}:{:<3}", cell.key_char, cell.note_name),
                CompactDensity::KeyOnly => format!(" {:>1}", cell.key_char),
            }
        } else {
            // Labels hidden: one illuminated glyph per pad, two columns wide.
            format!("{} ", pad_glyph(cell.state))
        };
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
    let quit_style = Style::default()
        .fg(palette.accent)
        .add_modifier(Modifier::BOLD);

    // Hints in priority order. The quit hint is reserved up front so it can
    // never be clipped off the right edge on a narrow terminal.
    const HINTS: [(&str, &str); 8] = [
        (" [↑/↓] ", "Octave"),
        (" [Tab] ", "Layout"),
        (" [F3] ", "Labels"),
        (" [F2] ", "Theme"),
        (" [Space] ", "Pedal"),
        (" [Enter] ", "Synth"),
        (" [←/→] ", "Transpose"),
        (" [PgUp/Dn] ", "Vel"),
    ];
    const QUIT: (&str, &str) = (" [Esc] ", "Quit");

    let available = usize::from(area.width.saturating_sub(2));
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut used = QUIT.0.chars().count() + QUIT.1.chars().count();

    for (key, label) in HINTS {
        // Key chip plus label plus the two separating spaces.
        let cost = key.chars().count() + label.chars().count() + 2;
        if used + cost > available {
            break;
        }
        used += cost;
        spans.push(Span::styled(key, key_style));
        spans.push(Span::raw(format!("{label}  ")));
    }
    spans.push(Span::styled(QUIT.0, quit_style));
    spans.push(Span::raw(QUIT.1));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(palette.accidental_fg));

    let paragraph = Paragraph::new(Line::from(spans))
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
                    text.contains("Catppuccin Mocha"),
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
    fn dodecagram_ring_requires_a_canvas_that_fits() {
        // The ring canvas is 23 columns by 9 rows, plus the complement line.
        assert!(dodecagram_shows_ring(Rect::new(0, 0, 25, 10)));
        assert!(dodecagram_shows_ring(Rect::new(0, 0, 48, 10)));
        // Too narrow for the 23-column canvas.
        assert!(!dodecagram_shows_ring(Rect::new(0, 0, 24, 10)));
        // Too short for nine canvas rows plus the complement summary.
        assert!(!dodecagram_shows_ring(Rect::new(0, 0, 48, 9)));
    }

    #[test]
    fn dodecagram_columns_keep_the_ring_a_fixed_size() {
        // Wide terminals pin the dodecagram to its canvas width so the chord
        // panel absorbs all extra space instead of the ring stretching out.
        let wide = dodecagram_columns(120);
        assert_eq!(wide[1], Constraint::Length(DODECA_PANEL_WIDTH));
        assert!(matches!(wide[0], Constraint::Min(_)));

        // Narrow terminals keep the proportional split so both panels survive.
        let narrow = dodecagram_columns(80);
        assert_eq!(narrow[0], Constraint::Percentage(60));
        assert_eq!(narrow[1], Constraint::Percentage(40));
    }

    #[test]
    fn bottom_panel_height_scales_with_terminal() {
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 120, 40)), 12);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 120, 26)), 10);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 80, 24)), 8);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 80, 20)), 5);
    }
}

/// Layout regressions for the Phase 2 chrome: these guard against fields being
/// sliced in half and against grid keys falling off the right edge.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod layout_regressions {
    use super::*;
    use crate::midi::MidiEngine;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn sample_app() -> App {
        let mut app = App::new(MidiEngine::silent(), None);
        // A C Maj7 voicing: C3, E3, G3, B3, with 'S' physically held.
        for note in [48u8, 52, 55, 59] {
            app.active_notes.insert(note);
        }
        app.active_coords.insert((1, 1));
        app.detected_chord = Some("C Maj7".to_string());
        app
    }

    fn frame_rows_for(app: &App, width: u16, height: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test backend");
        terminal.draw(|frame| render(frame, app)).expect("draw");
        let cells: Vec<String> = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol().to_string())
            .collect();
        cells
            .chunks(usize::from(width))
            .map(|row| row.concat())
            .collect()
    }

    fn frame_rows(width: u16, height: u16) -> Vec<String> {
        frame_rows_for(&sample_app(), width, height)
    }

    fn frame_text(width: u16, height: u16) -> String {
        frame_rows(width, height).concat()
    }

    fn frame_text_for(app: &App, width: u16, height: u16) -> String {
        frame_rows_for(app, width, height).concat()
    }

    #[test]
    fn compact_grid_shows_every_key_on_80x24() {
        let text = frame_text(80, 24);
        // The widest compact row must keep its rightmost pads visible.
        for key in ["=:F#5", "]:C#5", "':F#4", "/:B3"] {
            assert!(text.contains(key), "compact grid clipped {key}");
        }
        // Compact rows never spill past the grid's inner width. The buffer is
        // one flat string, so slice it back into 80-*character* rows first
        // (box-drawing glyphs are multi-byte in UTF-8).
        let cells: Vec<char> = text.chars().collect();
        for row in cells.chunks(80) {
            let row: String = row.iter().collect();
            if row.contains(":G#3") {
                assert!(
                    row.ends_with('\u{2502}'),
                    "row not bounded by the block: {row}"
                );
            }
        }
    }

    #[test]
    fn footer_keeps_quit_visible_and_theme_when_it_fits() {
        for (width, height) in [(120u16, 35u16), (80, 24), (60, 20), (40, 14)] {
            let rows = frame_rows(width, height);
            // The footer block is the last three rows: border, content, border.
            let footer = rows
                .get(rows.len().saturating_sub(2))
                .map_or("", String::as_str);
            assert!(
                footer.contains("Quit"),
                "quit hint missing at {width}x{height}: {footer:?}"
            );
            if width >= 80 {
                assert!(
                    footer.contains("Theme"),
                    "theme hint missing at {width}x{height}: {footer:?}"
                );
            }
        }
    }

    #[test]
    fn short_terminal_drops_panels_instead_of_mangling_the_footer() {
        // 12 rows cannot host header + grid + panels + footer; the harmonic
        // panels should step aside so the footer keeps all three of its rows.
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 40, 12)), 0);
        assert_eq!(bottom_panel_height(Rect::new(0, 0, 40, 14)), 0);

        let rows = frame_rows(40, 12);
        assert_eq!(rows.len(), 12);
        // Footer content row is still a real row, not a border squeeze.
        assert!(rows[10].contains("Quit"), "footer mangled: {:?}", rows[10]);
        // And the harmonic panels are gone rather than border-only.
        let text = rows.concat();
        assert!(!text.contains("Barcode"), "panels should be dropped");
    }

    #[test]
    fn header_drops_whole_fields_instead_of_clipping() {
        // Wide enough for every field, including the port name.
        let wide = frame_text(120, 35);
        assert!(wide.contains("Port: None"), "port field missing at 120x35");

        // Narrow: the tail is dropped as a field, and no value is left half-cut.
        let narrow = frame_text(80, 24);
        assert!(narrow.contains("Root: F2 (41)"), "root field lost at 80x24");
        assert!(!narrow.contains("Port:"), "port should be dropped at 80x24");
        assert!(!narrow.contains("Port: N\u{2502}") && !narrow.contains("Port: N\n"));
    }

    #[test]
    fn dodecagram_lines_are_not_doubled() {
        // A shallow segment stamped with one diagonal glyph per step used to
        // paint `╲╲` / `╱╱╱╱╱` runs. The per-step glyph choice must avoid that.
        let canvas = build_dodecagram_canvas(PitchClassSet::from_midi_notes(&[48, 52, 55, 59]));
        for row in &canvas {
            let text: String = row.iter().collect();
            assert!(!text.contains("╲╲"), "doubled backslash in row: {text:?}");
            assert!(!text.contains("╱╱"), "doubled slash in row: {text:?}");
        }
    }

    #[test]
    fn dodecagram_ring_is_drawn_without_a_chord() {
        // A single note has no polygon, but the perimeter must still render so
        // the widget reads as a dodecagram rather than a lone dot.
        let canvas = build_dodecagram_canvas(PitchClassSet::from_midi_notes(&[60]));
        let text: String = canvas.iter().flatten().collect();
        assert!(
            text.matches(DODECA_RING_CHAR).count() >= 24,
            "perimeter ring too sparse: {text:?}"
        );
        // Exactly one illuminated node for the single active pitch class.
        assert_eq!(text.matches('●').count(), 1);
        assert!(text.contains('●'), "active pitch class not marked");
    }

    #[test]
    fn dodecagram_chord_polygon_overrides_the_ring() {
        // C Maj7 must produce solid connecting lines over the dotted perimeter.
        let canvas = build_dodecagram_canvas(PitchClassSet::from_midi_notes(&[48, 52, 55, 59]));
        let text: String = canvas.iter().flatten().collect();
        assert!(
            ['─', '│', '╱', '╲'].iter().any(|g| text.contains(*g)),
            "chord polygon lines missing: {text:?}"
        );
        assert_eq!(text.matches('●').count(), 4, "expected four active nodes");
    }

    #[test]
    fn label_toggle_hides_grid_lettering() {
        let mut app = sample_app();

        // Expanded layout (120x40): labels on shows text inside each pad.
        app.show_labels = true;
        let expanded_on = frame_text_for(&app, 120, 40);
        assert!(
            expanded_on.contains("1 G#3"),
            "labelled pad text missing when labels are on"
        );

        // Labels off leaves the boxes empty — no lettering, no state glyph.
        app.show_labels = false;
        let expanded_off = frame_text_for(&app, 120, 40);
        assert!(
            !expanded_off.contains("G#3"),
            "pad lettering still present with labels off"
        );
        assert!(
            !expanded_off.contains('▪') && !expanded_off.contains('▫'),
            "expanded pads should be empty boxes when labels are hidden"
        );
        // The boxes themselves must still be drawn.
        assert!(
            expanded_off.contains("┌─────┐") && expanded_off.contains("└─────┘"),
            "pad borders missing when labels are hidden"
        );

        // Compact layout (80x24) has no boxes to leave empty, so it keeps a
        // single state glyph per pad to preserve the harmonic field.
        let compact_off = frame_text_for(&app, 80, 24);
        assert!(
            compact_off.contains('▪') || compact_off.contains('▫'),
            "compact pads should keep state glyphs when labels are hidden"
        );
    }
}
