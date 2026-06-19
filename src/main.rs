
mod app;
mod config;
mod file_browser;
mod palette;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, poll, read,
};
use crossterm::execute;
use ratatui::DefaultTerminal;

fn main() -> io::Result<()> {
            let _cfg = config::load();

    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
                ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = app::DrawingApp::new();

        if app::DrawingApp::session_exists() {
        app.show_startup_dialog = true;
        app.startup_dialog_idx = 0;
    }

            execute!(io::stdout(), EnableMouseCapture)?;

    while !app.should_quit {
        terminal.draw(|frame| app.render(frame))?;
                app.handle_event()?;
                        while poll(Duration::ZERO)? {
            let ev = read()?;
            app.handle_raw_event(&ev)?;
        }
    }

    execute!(io::stdout(), DisableMouseCapture)?;
    Ok(())
}
