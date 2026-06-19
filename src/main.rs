// Opendraw — a simple terminal drawing app built with Ratatui and Crossterm.
//
// Architecture overview:
//
//   main.rs          — Entry point. Initialises/restores the terminal, starts the
//                      render-event loop, and enables mouse capture.
//   palette.rs       — The colour palette: a list of 11 named colours and a
//                      current-selection index. Used to pick drawing colours.
//   app/             — Application module (directory with submodules):
//       mod.rs       — DrawingApp struct (all state), constructor, helpers,
//                      coordinate conversion, and the render() method that
//                      orchestrates every UI element.
//       draw.rs      — All drawing algorithms: brush stamp, erase, Bresenham
//                      line, rectangle/circle shapes, flood fill, gradient,
//                      symmetry mirroring.
//       event.rs     — Event dispatch: keyboard handling, mouse handling,
//                      file-browser sub-dialog events.
//       handlers.rs  — Popup/modal keyboard handlers and palette bar click.
//       history.rs   — Undo/redo stack management, save/load .txt files,
//                      PNG export via the `image` crate.
//       selection.rs — Copy, cut, paste, and nudge for rectangular selections.
//       session.rs   — Automatic session persistence (save/restore to disk).
//   ui/              — Rendering module (directory with submodules):
//       mod.rs       — Re-exports all public rendering functions.
//       col.rs       — Colour theme constants.
//       layout.rs    — Screen split into header/canvas/footer/status.
//       header.rs    — Tab bar, info bar, separator.
//       canvas.rs    — Canvas painting, grid, cursor preview, text overlay.
//       status.rs    — Pixel count, coordinates, colour history.
//       palette_bar.rs — Palette row and colour picker/selector/input popups.
//       popups.rs    — File browser, tab rename, help, startup, context menu,
//                      canvas resize dialogs.
//   file_browser.rs  — Directory navigation, file listing, and save/load dialog
//                      state. Used by the ^S (save) and ^O (open) commands.
//
// Data model:
//   The canvas is a BTreeMap<(u16, u16), ratatui::style::Color> — each
//   occupied pixel maps its coordinate to a colour. Empty spaces are simply
//   absent from the map. This is simpler and more memory-efficient than a
//   dense 2D array for sparse drawings, and BTreeMap gives sorted iteration
//   (useful for export).

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
    // Load/generate config early — before terminal init — so the config file
    // is created even if terminal init fails (e.g. in non-TTY environments).
    let _cfg = config::load();

    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    // On exit, ratatui::restore() restores the original terminal state:
    // cooked mode, main screen buffer, and — critically — disables mouse
    // capture automatically.
    ratatui::restore();
    result
}

/// The main loop: draw a frame, then block for the next event, repeat.
///
/// This loop structure (render → handle_event → loop) is the standard
/// Ratatui pattern:
///   1. `draw()` renders the current state to the terminal
///   2. `handle_event()` blocks for input and updates state
///      Both run on the same thread — no concurrency needed.
fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = app::DrawingApp::new();

    // If a previous session exists, show the startup dialog.
    if app::DrawingApp::session_exists() {
        app.show_startup_dialog = true;
        app.startup_dialog_idx = 0;
    }

    // Mouse capture is separate from raw mode. We enable it here and disable
    // manually after the loop (ratatui::restore does NOT disable mouse capture).
    execute!(io::stdout(), EnableMouseCapture)?;

    while !app.should_quit {
        terminal.draw(|frame| app.render(frame))?;
        // Block until the first event arrives, then handle it.
        app.handle_event()?;
        // Drain any additional pending events (e.g. rapid mouse moves)
        // without re-rendering between each one — only the final state matters.
        while poll(Duration::ZERO)? {
            let ev = read()?;
            app.handle_raw_event(&ev)?;
        }
    }

    execute!(io::stdout(), DisableMouseCapture)?;
    Ok(())
}
