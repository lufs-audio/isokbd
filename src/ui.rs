//! Terminal User Interface rendering with Ratatui.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::layout::{calculate_midi_note, ALL_ROWS};
use crate::theory::{note_to_name, pitch_class_name};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header bar
            Constraint::Min(12),    // Keyboard grid
            Constraint::Length(4),  // Chord / Status panel
            Constraint::Length(3),  // Navigation / shortcuts footer
        ])
        .split(size);

    let (header_area, grid_area, chord_area, footer_area) = match chunks.as_ref() {
        [h, g, c, f] => (*h, *g, *c, *f),
        _ => return,
    };

    render_header(frame, header_area, app);
    render_keyboard_grid(frame, grid_area, app);
    render_chord_panel(frame, chord_area, app);
    render_footer(frame, footer_area);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let root_str = note_to_name(app.root_note);
    let sustain_color = if app.sustain_enabled {
        Color::Green
    } else {
        Color::DarkGray
    };

    let header_line = Line::from(vec![
        Span::styled(" ISOKBD ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" │ Mode: "),
        Span::styled(app.layout_mode.name(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" │ Root: "),
        Span::styled(format!("{root_str} ({})", app.root_note), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(" │ Vel: "),
        Span::styled(format!("{}", app.velocity), Style::default().fg(Color::White)),
        Span::raw(" │ Sustain: "),
        Span::styled(if app.sustain_enabled { "ON" } else { "OFF" }, Style::default().fg(sustain_color).add_modifier(Modifier::BOLD)),
        Span::raw(" │ Port: "),
        Span::styled(app.midi.port_name(), Style::default().fg(Color::Green)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(header_line).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn render_keyboard_grid(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Isomorphic QWERTY Grid ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Stagger offsets in characters for rows 3, 2, 1, 0
    let row_staggers = [6, 4, 2, 0]; // Visual stagger for QWERTY rows

    let mut lines = Vec::new();

    // Render from row 3 (Number row) down to row 0 (Bottom letter row)
    for (row_idx, &keys) in ALL_ROWS.iter().enumerate().rev() {
        let stagger_spaces = row_staggers.get(row_idx).copied().unwrap_or(0);
        let mut spans = Vec::new();

        // Stagger spacing
        if stagger_spaces > 0 {
            spans.push(Span::raw(" ".repeat(stagger_spaces)));
        }

        for key in keys {
            let is_active = app.active_coords.contains_key(&(row_idx, key.col));
            let midi_note = calculate_midi_note(row_idx, key.col, app.root_note, app.layout_mode);
            let note_str = midi_note.map(note_to_name).unwrap_or_else(|| "--".to_string());

            let is_root = midi_note.map(|n| n % 12 == app.root_note % 12).unwrap_or(false);
            let is_black_key = midi_note.map(|n| pitch_class_name(n).contains('#')).unwrap_or(false);

            let style = if is_active {
                // Key currently held
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_root {
                // Root note highlight
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_black_key {
                // Accidental
                Style::default().fg(Color::DarkGray)
            } else {
                // Natural note
                Style::default().fg(Color::White)
            };

            // Format cell e.g. "[ Z C3 ]"
            let cell_text = format!(" [{:>1} {:<3}]", key.label, note_str);
            spans.push(Span::styled(cell_text, style));
        }

        lines.push(Line::from(spans));
        // Add vertical spacing between rows
        if row_idx > 0 {
            lines.push(Line::raw(""));
        }
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    frame.render_widget(paragraph, inner);
}

fn render_chord_panel(frame: &mut Frame, area: Rect, app: &App) {
    let chord_title = app.detected_chord.as_deref().unwrap_or("No notes playing");
    let active_notes_str: Vec<String> = app.active_notes.iter().map(|&n| note_to_name(n)).collect();
    let notes_summary = if active_notes_str.is_empty() {
        "—".to_string()
    } else {
        active_notes_str.join(", ")
    };

    let content = vec![
        Line::from(vec![
            Span::styled("Chord: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                chord_title,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Active Notes: ", Style::default().fg(Color::DarkGray)),
            Span::styled(notes_summary, Style::default().fg(Color::White)),
            if let Some(ref msg) = app.status_message {
                Span::styled(format!("  •  {msg}"), Style::default().fg(Color::Cyan))
            } else {
                Span::raw("")
            },
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(content).block(block);
    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer_line = Line::from(vec![
        Span::styled(" [↑/↓] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Octave  "),
        Span::styled(" [←/→] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Transpose  "),
        Span::styled(" [Tab] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Layout  "),
        Span::styled(" [Space] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Sustain  "),
        Span::styled(" [PgUp/PgDn] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Vel  "),
        Span::styled(" [Esc/Ctrl+C] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("Quit"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(footer_line).block(block).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
