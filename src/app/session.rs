

use std::{fs, io};

use ratatui::layout::Position;
use ratatui::style::Color;

use crate::app::draw::color_to_rgb;
use crate::app::DrawingApp;

fn session_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::PathBuf::from(&home).join(".local/share/pixdraw");
    let _ = fs::create_dir_all(&dir);
    dir.join("session.dat")
}


fn color_to_rgb_str(c: &Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("{r},{g},{b}"),
        _ => {
            let (r, g, b) = color_to_rgb(*c);
            format!("{r},{g},{b}")
        }
    }
}

fn parse_color_list(s: &str) -> Vec<Color> {
    let parts: Vec<&str> = s.splitn(2, '|').collect::<Vec<_>>();
    if parts.len() < 2 { return Vec::new(); }
    let _count = parts[0].parse::<usize>().unwrap_or(0);
    parts[1].split('|').filter_map(parse_single_color).collect()
}

fn parse_single_color(s: &str) -> Option<Color> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 3
        && let (Ok(r), Ok(g), Ok(b)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>(), parts[2].parse::<u8>())
    {
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}


impl DrawingApp {
        pub fn delete_session() {
        let _ = fs::remove_file(session_path());
    }

        pub fn session_exists() -> bool {
        session_path().exists()
    }
}


impl DrawingApp {
                pub fn save_session(&self) -> io::Result<()> {
        let mut out = String::new();
                out.push_str(&format!("brush_size:{}\n", self.brush_size));
        out.push_str(&format!("symmetry:{}\n", self.symmetry_mode as u8));
        out.push_str(&format!("grid:{}\n", self.show_grid as u8));
        out.push_str(&format!("palette_idx:{}\n", self.palette.index));
        out.push_str(&format!("current_tab:{}\n", self.current_tab));
        out.push_str(&format!("canvas_width:{}\n", self.canvas_width));
        out.push_str(&format!("canvas_height:{}\n", self.canvas_height));
        if !self.custom_colors.is_empty() {
            let parts: Vec<String> = self.custom_colors.iter().map(color_to_rgb_str).collect();
            out.push_str(&format!("custom_colors:{}|{}\n", self.custom_colors.len(), parts.join("|")));
        }
        if !self.color_history.is_empty() {
            let parts: Vec<String> = self.color_history.iter().map(color_to_rgb_str).collect();
            out.push_str(&format!("color_history:{}|{}\n", self.color_history.len(), parts.join("|")));
        }
        if let Some(c) = &self.custom_color_override {
            out.push_str(&format!("color_override:{}\n", color_to_rgb_str(c)));
        }
        out.push_str(&format!("color_gen_seed:{}\n", self.color_gen_seed));
                for tab in &self.tabs {
            out.push_str(&format!("tab_name:{}\n", tab.name));
            for (pos, text) in &tab.text_entries {
                out.push_str(&format!("text:{}|{}|{}\n", pos.x, pos.y, text));
            }
            for (&(x, y), color) in &tab.points {
                if let Color::Rgb(r, g, b) = color {
                    out.push_str(&format!("point:{}|{}|{}|{}|{}\n", x, y, r, g, b));
                } else {
                    let (r, g, b) = color_to_rgb(*color);
                    out.push_str(&format!("point:{}|{}|{}|{}|{}\n", x, y, r, g, b));
                }
            }
        }
        fs::write(session_path(), out)
    }

            pub fn restore_session(&mut self) -> io::Result<bool> {
        let path = session_path();
        if !path.exists() {
            return Ok(false);
        }
        let data = fs::read_to_string(&path)?;
        let mut lines_iter = data.lines().peekable();

                let mut saved_tab: Option<usize> = None;

                while let Some(line) = lines_iter.peek() {
            if line.starts_with("tab_name:") {
                break;
            }
            let line = lines_iter.next().unwrap();
            if let Some(rest) = line.strip_prefix("brush_size:") {
                if let Ok(n) = rest.parse::<u16>() { self.brush_size = n.clamp(1, 5); }
            } else if let Some(rest) = line.strip_prefix("symmetry:") {
                self.symmetry_mode = rest == "1";
            } else if let Some(rest) = line.strip_prefix("grid:") {
                self.show_grid = rest == "1";
            } else if let Some(rest) = line.strip_prefix("palette_idx:") {
                if let Ok(n) = rest.parse::<usize>() { self.palette.select(n); }
            } else if let Some(rest) = line.strip_prefix("current_tab:") {
                                if let Ok(n) = rest.parse::<usize>() { saved_tab = Some(n); }
            } else if let Some(rest) = line.strip_prefix("canvas_width:") {
                if let Ok(n) = rest.parse::<u16>() { self.canvas_width = n; }
            } else if let Some(rest) = line.strip_prefix("canvas_height:") {
                if let Ok(n) = rest.parse::<u16>() { self.canvas_height = n; }
            } else if let Some(rest) = line.strip_prefix("custom_colors:") {
                self.custom_colors = parse_color_list(rest);
            } else if let Some(rest) = line.strip_prefix("color_history:") {
                for c in parse_color_list(rest) {
                    self.color_history.push_back(c);
                    if self.color_history.len() > 5 { self.color_history.pop_front(); }
                }
            } else if let Some(rest) = line.strip_prefix("color_override:") {
                self.custom_color_override = parse_single_color(rest);
            } else if let Some(rest) = line.strip_prefix("color_gen_seed:")
                && let Ok(n) = rest.parse::<u64>() { self.color_gen_seed = n; }
        }

                self.tabs.clear();
        self.tabs.push(crate::app::TabData::new("Untitled".to_string()));
        self.current_tab = 0;

        let mut current_tab_idx = 0usize;
        while lines_iter.peek().is_some() {
            let line = lines_iter.next().unwrap();
            if let Some(name) = line.strip_prefix("tab_name:") {
                if current_tab_idx == 0 {
                    self.tabs[0].name = name.to_string();
                } else {
                    self.tabs.push(crate::app::TabData::new(name.to_string()));
                }
                current_tab_idx = self.tabs.len() - 1;
            } else if let Some(rest) = line.strip_prefix("text:") {
                let parts: Vec<&str> = rest.splitn(3, '|').collect();
                if parts.len() == 3
                    && let (Ok(x), Ok(y)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>())
                {
                    self.tabs[current_tab_idx].text_entries.push((Position::new(x, y), parts[2].to_string()));
                }
            } else if let Some(rest) = line.strip_prefix("point:") {
                let parts: Vec<&str> = rest.split('|').collect();
                if parts.len() == 5
                    && let (Ok(x), Ok(y), Ok(r), Ok(g), Ok(b)) = (
                        parts[0].parse::<u16>(), parts[1].parse::<u16>(),
                        parts[2].parse::<u8>(), parts[3].parse::<u8>(), parts[4].parse::<u8>(),
                    )
                {
                    self.tabs[current_tab_idx].points.insert((x, y), Color::Rgb(r, g, b));
                }
            }
        }

                if let Some(n) = saved_tab {
            self.current_tab = n.min(self.tabs.len().saturating_sub(1));
        }

        self.custom_cycle_idx = self.custom_colors.len().saturating_sub(1);
        if self.current_tab >= self.tabs.len() {
            self.current_tab = 0;
        }
        Ok(true)
    }
}
