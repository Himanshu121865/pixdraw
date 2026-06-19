// ── ui/mod.rs ────────────────────────────────────────────────────────
// Re-export all public UI rendering functions so the `app` module (and
// anyone else) can call `ui::layout()`, `ui::render_header()`, etc.
//
// Why a re-export module?
//   Each rendering function lives in its own file (for organisation),
//   but callers outside the `ui` module don't need to know the file
//   structure. The re-exports provide a flat namespace: `ui::layout`
//   instead of `ui::layout::layout`.

mod col;
mod layout;
mod header;
mod canvas;
mod status;
mod palette_bar;
mod popups;
mod help_popup;
mod dialog;

pub(crate) use col::init_from_config;

pub use layout::layout;
pub use header::render_header;
pub use canvas::render_canvas;
pub use canvas::render_grid;
pub use canvas::render_cursor_preview;
pub use canvas::render_text_overlay;
pub use status::render_status_bar;
pub use palette_bar::render_palette_bar;
pub use palette_bar::render_color_picker;
pub use palette_bar::render_color_selector;
pub use palette_bar::render_color_input;
pub use popups::render_file_browser;
pub use help_popup::render_help_popup;
pub use dialog::render_tab_rename_dialog;
pub use dialog::render_startup_dialog;
pub use dialog::render_context_menu;
pub use dialog::render_canvas_resize_dialog;
