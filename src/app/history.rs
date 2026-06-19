// ── app/history.rs ──────────────────────────────────────────────────
// Undo/redo stack management, save/load from .txt files, and PNG export.
//
// Undo/redo works by storing full clones of the `points` BTreeMap. This is
// simple and correct, but memory-hungry for large drawings.
//
// LIMITATION: text_entries are NOT tracked by undo/redo. Only the pixel
// map (BTreeMap) is snapshotted. This means undoing a clear will restore
// pixels but NOT text that was placed before the clear. A proper fix would
// require storing a combined snapshot struct with both `points` and
// `text_entries`.
//
// Save format:
//   P <x> <y> <palette_index>    — one line per pixel
//   T <x> <y> <text>             — one line per text entry
//
// Load reverses the process: "P" lines restore pixels via palette index,
// "T" lines restore text entries.

use std::{fs, io, path::Path};

use ratatui::layout::Position;
use ratatui::style::Color;

use crate::app::{draw::color_to_rgba, DrawingApp};

// ── Session persistence now lives in `session.rs` ──────────────────

impl DrawingApp {
    // ── Undo / Redo ──────────────────────────────────────────────
    // `history` stores previous `points` states. `redo_stack` stores
    // states that were undone. Both are LIFO (Vec::push / Vec::pop).
    //
    // `push_history()` is called BEFORE any mutation. After pushing,
    // the redo stack is cleared because a new branch has been created
    // (any previous redos are no longer reachable).

    /// Save the current `points` state to the undo stack.
    pub(crate) fn push_history(&mut self) {
        let pts = self.points.clone();
        self.history.push(pts);
        self.redo_stack.clear();
    }

    /// Restore the most recent history entry. The current state is
    /// pushed onto the redo stack so it can be recovered.
    pub(crate) fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            let replaced = std::mem::replace(&mut self.points, prev);
            self.redo_stack.push(replaced);
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let replaced = std::mem::replace(&mut self.points, next);
            self.history.push(replaced);
        }
    }

    // ── Save / Load ──────────────────────────────────────────────

    /// Serialise the canvas to a .txt file.
    /// Format:
    ///   "P x y palette_index" — palette colours (compact)
    ///   "R x y r g b"        — arbitrary RGB colours (exact)
    ///   "T x y text"         — text entries
    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        let mut out = String::new();
        for (&(x, y), &color) in &self.points {
            if let Some(idx) = self.palette.colors.iter().position(|(c, _)| *c == color) {
                out.push_str(&format!("P {} {} {}\n", x, y, idx));
            } else if let ratatui::style::Color::Rgb(r, g, b) = color {
                out.push_str(&format!("R {} {} {} {} {}\n", x, y, r, g, b));
            }
        }
        for (pos, text) in &self.text_entries {
            out.push_str(&format!("T {} {} {}\n", pos.x, pos.y, text));
        }
        fs::write(path, out)
    }

    /// Deserialise a .txt or image file (.jpg, .png, etc.).
    /// For .txt files, parses "P x y idx" lines.
    /// For image files, loads, resizes to fit canvas, and quantizes to
    /// the 14-colour palette using LAB perceptual matching.
    /// After loading an image, `canvas_width` / `canvas_height` are set
    /// to the image dimensions so the virtual canvas matches the image.
    pub fn load_from(&mut self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let is_image = matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp");

        if is_image {
            let img = image::open(path).map_err(io::Error::other)?;
            let (w, h) = (img.width(), img.height());

            // ── Sizing ────────────────────────────────────────────
            // Fit the image within the available canvas area while
            // maintaining aspect ratio. Landscape images fill the
            // width; portrait images fill the height. This ensures
            // wide images span the full canvas and tall images aren't
            // cut off at the bottom.
            let (tw, th) = crossterm::terminal::size()
                .unwrap_or((187, 46));
            let max_canvas_w = (tw as u32).saturating_sub(2).max(8);
            let max_canvas_h = (th as u32).saturating_sub(7).max(4);

            let (nw, nh) = if w >= h {
                // Landscape: fill width, cap height to visible area
                let nw = max_canvas_w;
                let nh = (nw as f64 * h as f64 / w as f64).round().max(1.0) as u32;
                (nw, nh.min(max_canvas_h))
            } else {
                // Portrait: fill height, cap width to visible area
                let nh = max_canvas_h;
                let nw = (nh as f64 * w as f64 / h as f64).round().max(1.0) as u32;
                (nw.min(max_canvas_w), nh)
            };

            let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
            let rgb = resized.to_rgb8();

            self.push_history();
            self.points.clear();
            self.text_entries.clear();

            // ── Centring ──────────────────────────────────────────
            // Compute (off_x, off_y) so the image is centred within
            // the current virtual canvas. If no canvas size is set
            // (0,0), centre within the terminal body area.
            let (cw, ch) = if self.canvas_width > 0 {
                (self.canvas_width as u32, self.canvas_height.max(1) as u32)
            } else {
                (max_canvas_w, max_canvas_h)
            };
            let off_x = (cw.saturating_sub(nw) / 2) as u16;
            let off_y = (ch.saturating_sub(nh) / 2) as u16;

            for y in 0..nh {
                for x in 0..nw {
                    let p = rgb.get_pixel(x, y);
                    self.points.insert((x as u16 + off_x, y as u16 + off_y), Color::Rgb(p[0], p[1], p[2]));
                }
            }
            self.canvas_width = cw as u16;
            self.canvas_height = ch as u16;
        } else {
            let data = fs::read_to_string(path)?;
            self.push_history();
            self.points.clear();
            self.text_entries.clear();

            for line in data.lines() {
                if let Some(rest) = line.strip_prefix('P') {
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if parts.len() == 3
                        && let (Ok(x), Ok(y), Ok(idx)) =
                            (parts[0].parse(), parts[1].parse(), parts[2].parse::<usize>())
                            && idx < self.palette.colors.len() {
                                let color = self.palette.colors[idx].0;
                                self.points.insert((x, y), color);
                            }
                } else if let Some(rest) = line.strip_prefix('R') {
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if parts.len() == 5
                        && let (Ok(x), Ok(y), Ok(r), Ok(g), Ok(b)) =
                            (parts[0].parse(), parts[1].parse(),
                             parts[2].parse(), parts[3].parse(), parts[4].parse())
                    {
                        self.points.insert((x, y), Color::Rgb(r, g, b));
                    }
                } else if let Some(rest) = line.strip_prefix('T') {
                    let parts: Vec<&str> = rest.trim().splitn(3, ' ').collect();
                    if parts.len() == 3
                        && let (Ok(x), Ok(y)) = (parts[0].parse(), parts[1].parse()) {
                            self.text_entries
                                .push((Position::new(x, y), parts[2].to_string()));
                        }
                }
            }
        }
        Ok(())
    }



    // ── PNG Export ────────────────────────────────────────────────
    // Uses the `image` crate to write an RGBA PNG. Palette colours are
    // mapped to approximate RGB via a match. The image is cropped to the
    // bounding box of all pixels, with a 1-pixel transparent border.

    /// Build an RgbaImage from the current canvas pixels.
    /// Uses canvas_width/canvas_height if set, otherwise crops to bounding box.
    /// Returns (image, width, height) or None if the canvas is empty.
    fn build_png_image(&self) -> Option<(image::RgbaImage, u32, u32)> {
        if self.points.is_empty() {
            return None;
        }

            let (out_w, out_h) = if self.canvas_width > 0 && self.canvas_height > 0 {
                (self.canvas_width as u32, self.canvas_height as u32)
            } else {
                let mut min_x = u16::MAX;
                let mut min_y = u16::MAX;
                let mut max_x = 0u16;
                let mut max_y = 0u16;
                for &(x, y) in self.points.keys() {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
                ((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32)
            };

        let mut img = image::RgbaImage::new(out_w, out_h);
        for (&(x, y), &color) in &self.points {
            if x < out_w as u16 && y < out_h as u16 {
                img.put_pixel(x as u32, y as u32, color_to_rgba(color));
            }
        }
        Some((img, out_w, out_h))
    }

    /// Export the canvas to a PNG file at the given path.
    pub fn export_png_to(&self, path: &std::path::Path) -> io::Result<()> {
        let Some((img, _w, _h)) = self.build_png_image() else {
            return Ok(());
        };
        img.save(path).map_err(io::Error::other)?;
        Ok(())
    }

    // ── Quick export ──────────────────────────────────────────────
    // Saves the canvas to opendraw_export.png so the user can open
    // it in an image viewer. Simpler than Ctrl+E (no file browser).

}
