//! Entrypoint for `isokbd`: an isomorphic QWERTY MIDI instrument TUI.

mod app;
mod layout;
mod midi;
mod theory;
mod ui;

use std::io::stdout;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use midi::MidiEngine;

fn main() -> Result<()> {
    // Set up crash / panic hook so the terminal state is ALWAYS restored
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            PopKeyboardEnhancementFlags,
            LeaveAlternateScreen
        );
        default_panic(panic_info);
    }));

    // Initialize MIDI Output
    let midi_engine = MidiEngine::new().context("Could not initialize MIDI engine")?;

    // Initialize terminal in raw mode with keyboard enhancements (for KeyRelease events)
    enable_raw_mode().context("Failed to enable terminal raw mode")?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    ).context("Failed to configure terminal enhancements")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create Ratatui terminal")?;

    let mut app = App::new(midi_engine);
    let tick_rate = Duration::from_millis(16); // ~60 FPS update rate

    while !app.should_quit {
        terminal.draw(|frame| {
            ui::render(frame, &app);
        }).context("Failed to draw TUI frame")?;

        if event::poll(tick_rate).context("Failed to poll terminal events")?
            && let Event::Key(key_event) = event::read().context("Failed to read terminal event")?
        {
            app.handle_key_event(key_event);
        }
    }

    // Clean teardown
    app.all_notes_off();
    execute!(
        terminal.backend_mut(),
        PopKeyboardEnhancementFlags,
        LeaveAlternateScreen
    ).context("Failed to restore terminal screen")?;
    disable_raw_mode().context("Failed to disable raw mode")?;

    Ok(())
}
