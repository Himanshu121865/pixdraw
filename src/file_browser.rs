// ── file_browser.rs ───────────────────────────────────────────────
// A directory-listing file browser used for Save (Ctrl+S) and Open
// (Ctrl+O) dialogs. It lists files in the current directory, supports
// navigation with keyboard/mouse, and distinguishes Save vs Load modes.
//
// How it works:
//   1. `open(mode)` sets `active = true`, starts at the CWD, and calls
//      `refresh()` to scan entries.
//   2. Entries are grouped: directories first (sorted), then files
//      (sorted). In Load mode, only loadable files (.txt, .jpg, .png,
//      .gif, .bmp) are shown. In Save mode, all files are shown.
//   3. Hidden files (names starting with `.`) are excluded to reduce
//      visual noise — the user probably doesn't need .gitignore when
//      browsing for pixel art.
//   4. The event loop in event.rs intercepts keyboard/mouse and
//      delegates to the browser's navigation methods. The browser
//      itself is stateless — it only tracks position and scrolling.
//
// Why a custom browser instead of a library?
//   A custom file browser keeps the save/load UX consistent with the
//   rest of the app (same colour theme, same input handling). A real
//   terminal file dialog would pull in a heavy dependency and fight
//   with our rendering architecture.

use std::{fs, path::PathBuf};

/// Whether the browser was opened for saving or loading.
#[derive(Clone, Copy, PartialEq)]
pub enum FileBrowserMode {
    Save,
    Load,
    ExportPng,
}

/// A single entry in the directory listing.
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

/// The file browser's full state: open/closed flag, mode, entry list,
/// selection cursor, scroll offset, current directory, and the
/// filename text input (used in Save mode).
pub struct FileBrowser {
    /// Whether the file browser popup is currently shown.
    pub active: bool,
    /// Whether we are in Save mode or Load mode.
    pub mode: FileBrowserMode,
    /// The listed files and directories in the current directory.
    pub entries: Vec<FileEntry>,
    /// Index into `entries` of the currently highlighted item.
    pub selected: usize,
    /// The directory being browsed.
    pub current_path: PathBuf,
    /// Last directory that was browsed (persists across opens).
    pub last_dir: PathBuf,
    /// Scrolling offset into `entries` (for the viewport).
    pub scroll_offset: usize,
    /// Text typed into the filename input (Save/ExportPng mode).
    pub filename_input: String,
    /// Whether the filename input has focus (i toggles it).
    pub filename_input_active: bool,
    /// One-shot message (e.g. "Cannot read directory").
    pub message: String,
}

impl FileBrowser {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            active: false,
            mode: FileBrowserMode::Load,
            entries: Vec::new(),
            selected: 0,
            current_path: cwd.clone(),
            last_dir: cwd,
            scroll_offset: 0,
            filename_input: String::new(),
            filename_input_active: false,
            message: String::new(),
        }
    }

    /// Open the browser in the given mode and refresh the file listing.
    /// Starts in the last browsed directory rather than CWD.
    pub fn open(&mut self, mode: FileBrowserMode) {
        self.mode = mode;
        self.active = true;
        self.filename_input.clear();
        self.filename_input_active = false;
        self.message.clear();
        self.current_path = self.last_dir.clone();
        self.refresh();
    }

    /// Re-read the current directory and rebuild the entry list.
    /// Directories come first (sorted), then files (sorted).
    pub fn refresh(&mut self) {
        self.entries.clear();
        let Ok(rd) = fs::read_dir(&self.current_path) else {
            self.message = "Cannot read directory".to_string();
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in rd.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files (starting with `.`)
            if name.starts_with('.') { continue; }

            if ft.is_dir() {
                dirs.push(FileEntry { name, is_dir: true });
            } else if self.mode == FileBrowserMode::Load {
                let is_img = name.ends_with(".txt")
                    || name.ends_with(".jpg") || name.ends_with(".jpeg")
                    || name.ends_with(".png")
                    || name.ends_with(".gif")
                    || name.ends_with(".bmp");
                if is_img {
                    files.push(FileEntry { name, is_dir: false });
                }
            } else if self.mode == FileBrowserMode::Save {
                // In Save mode show all files — the user may overwrite any file.
                files.push(FileEntry { name, is_dir: false });
            }
        }

        // Alphabetical sort ensures predictable ordering.
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

        // ".." entry — only if there is a parent directory.
        if self.current_path.parent().is_some() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                is_dir: true,
            });
        }

        self.entries.extend(dirs);
        self.entries.extend(files);

        // Reset selection when refreshing.
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Resolve the selected entry to a full filesystem path.
    /// For ".." returns the parent directory path.
    pub fn selected_path(&self) -> Option<PathBuf> {
        self.entries.get(self.selected).map(|e| {
            if e.name == ".." {
                self.current_path.parent().unwrap_or(&self.current_path).to_path_buf()
            } else {
                self.current_path.join(&e.name)
            }
        })
    }

    /// Move the selection cursor up (toward index 0).
    /// Adjusts scroll offset so the selected item stays visible
    /// (simple viewport lock — keeps selected within the window).
    pub fn navigate_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    /// Move the selection cursor down.
    pub fn navigate_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
            let max_visible = 18usize;  // arbitrary, matches popup_lines in ui
            if self.selected >= self.scroll_offset + max_visible {
                self.scroll_offset = self.selected - max_visible + 1;
            }
        }
    }

    /// "Enter" the selected entry. If it is a directory, navigate into it
    /// and return false. If it is a file, return true.
    pub fn enter_selected(&mut self) -> bool {
        let Some(entry) = self.entries.get(self.selected) else {
            return false;
        };

        if entry.is_dir {
            let new_path = if entry.name == ".." {
                self.current_path.parent().unwrap_or(&self.current_path).to_path_buf()
            } else {
                self.current_path.join(&entry.name)
            };

            if new_path.is_dir() {
                self.current_path = new_path;
                self.last_dir = self.current_path.clone();
                self.refresh();
            }
            false  // directory entered, still browsing
        } else {
            // File selected — remember the directory for next time.
            self.last_dir = self.current_path.clone();
            true   // file selected
        }
    }

    /// Navigate to the parent directory.
    pub fn go_up_dir(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.last_dir = self.current_path.clone();
            self.refresh();
        }
    }
}
