mod app;
mod audio;
mod dsp;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> Result<()> {
    // 1. Setup panic hook for safe terminal restoration
    setup_panic_hook();

    // 2. Initialize terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Initialize application state and audio subsystem
    let mut app = App::new()?;

    // 4. Main loop @ 60 FPS
    let tick_rate = Duration::from_millis(16);
    let res = Ok(());

    while app.running {
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key_event(key);
            }
        }

        app.update();
    }

    // 5. Safe cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}
