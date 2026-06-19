
use std::{fs, path::PathBuf};

#[derive(Clone, Copy, PartialEq)]
pub enum FileBrowserMode {
    Save,
    Load,
    ExportPng,
}

pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
}

pub struct FileBrowser {
        pub active: bool,
        pub mode: FileBrowserMode,
        pub entries: Vec<FileEntry>,
        pub selected: usize,
        pub current_path: PathBuf,
        pub last_dir: PathBuf,
        pub scroll_offset: usize,
        pub filename_input: String,
        pub filename_input_active: bool,
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

            pub fn open(&mut self, mode: FileBrowserMode) {
        self.mode = mode;
        self.active = true;
        self.filename_input.clear();
        self.filename_input_active = false;
        self.message.clear();
        self.current_path = self.last_dir.clone();
        self.refresh();
    }

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
                                files.push(FileEntry { name, is_dir: false });
            }
        }

                dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

                if self.current_path.parent().is_some() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                is_dir: true,
            });
        }

        self.entries.extend(dirs);
        self.entries.extend(files);

                self.selected = 0;
        self.scroll_offset = 0;
    }

            pub fn selected_path(&self) -> Option<PathBuf> {
        self.entries.get(self.selected).map(|e| {
            if e.name == ".." {
                self.current_path.parent().unwrap_or(&self.current_path).to_path_buf()
            } else {
                self.current_path.join(&e.name)
            }
        })
    }

                pub fn navigate_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

        pub fn navigate_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
            let max_visible = 18usize;              if self.selected >= self.scroll_offset + max_visible {
                self.scroll_offset = self.selected - max_visible + 1;
            }
        }
    }

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
            false          } else {
                        self.last_dir = self.current_path.clone();
            true           }
    }

        pub fn go_up_dir(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.last_dir = self.current_path.clone();
            self.refresh();
        }
    }
}
