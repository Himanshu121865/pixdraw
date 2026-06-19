// ── app/draw.rs ────────────────────────────────────────────────────
// All drawing algorithms: brush stamping, erasing, Bresenham line,
// rectangle/circle shapes, flood fill, gradient fill, symmetry
// mirroring, and the eyedropper colour picker.
//
// Each method is `pub(crate)` so that sibling modules (event.rs) can
// call them, but external modules cannot.
//
// Coordinate system:
//   All drawing operates in canvas-local coordinates where (0,0) is
//   the top-left pixel inside the canvas border. Coordinates are
//   stored as (u16, u16) tuples — they are never negative.
//
//   `wrapping_add_signed` is used for offset calculations near the
//   canvas edge. If the result would underflow (negative), it wraps
//   around to a large u16 — but the pixel is only inserted into the
//   BTreeMap if the user sees it, and out-of-bounds clicks are caught
//   by `local_canvas_position()` which returns None.

use ratatui::{
    layout::Position,
    style::Color,
};

use crate::app::DrawingApp;

// ── Colour conversion helpers ──────────────────────────────────────
// Ratatui's Color enum has named variants (Color::Red, Color::LightBlue,
// etc.) plus Color::Rgb(r, g, b). PNG export needs concrete RGBA bytes,
// and gradient fill interpolates between two colours as RGB floats.
// These two helpers bridge the gap.

/// Convert a ratatui Color to an `image::Rgba<u8>` for PNG export.
/// Named colours are mapped to approximate RGB values via match.
pub(crate) fn color_to_rgba(c: Color) -> image::Rgba<u8> {
    match c {
        Color::Rgb(r, g, b) => image::Rgba([r, g, b, 255]),
        Color::Indexed(i) => {
            let idx = i as usize % crate::palette::PALETTE.len();
            color_to_rgba(crate::palette::PALETTE[idx].0)
        }
        _ => {
            let (r, g, b) = color_to_rgb(c);
            image::Rgba([r, g, b, 255])
        }
    }
}

/// Convert a named or RGB Color to a plain (u8, u8, u8) triplet.
/// Used internally by `lerp_color` for gradient interpolation.
///
/// Named colours don't store RGB values — they're just enum variants.
/// We match on `format!("{:?}")` to get the variant name as a string,
/// then map to hardcoded RGB approximations. This is not exhaustive
/// but covers the standard terminal colour set.
pub(crate) fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => {
            // format!("{:?}") gives the variant name as a string e.g. "Red" "LightBlue"
            let s = format!("{:?}", c);
            match s.as_str() {
                "Black" => (0, 0, 0),
                "Red" => (255, 0, 0),
                "Green" => (0, 128, 0),
                "Yellow" => (255, 255, 0),
                "Blue" => (0, 0, 255),
                "Magenta" => (255, 0, 255),
                "Cyan" => (0, 255, 255),
                "White" => (255, 255, 255),
                "DarkGray" => (128, 128, 128),
                "LightRed" => (255, 96, 96),
                "LightGreen" => (96, 255, 96),
                "LightYellow" => (255, 255, 96),
                "LightBlue" => (96, 96, 255),
                "LightMagenta" => (255, 96, 255),
                "LightCyan" => (96, 255, 255),
                _ => (255, 255, 255),
            }
        }
    }
}

// ── Brush / Erase / Symmetry ───────────────────────────────────────

impl DrawingApp {
    /// Stamp a filled circle of the current brush size at `pos`.
    /// brush_size 1 → single pixel. brush_size 5 → radius 4 circle.
    /// Uses `wrapping_add_signed` to safely handle coordinates near 0.
    ///
    /// The stamp is always a circle, not a square. The radius is
    /// brush_size - 1, so brush 1 = point (radius 0), brush 2 = radius 1, etc.
    pub(crate) fn stamp_brush(&mut self, pos: (u16, u16), color: Color) {
        let r = self.brush_size.saturating_sub(1) as i16;
        if r == 0 {
            self.points.insert(pos, color);
            return;
        }
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    self.points.insert(
                        (pos.0.wrapping_add_signed(dx), pos.1.wrapping_add_signed(dy)),
                        color,
                    );
                }
            }
        }
    }

    /// Like stamp_brush, but removes pixels instead of adding them.
    pub(crate) fn stamp_erase(&mut self, pos: (u16, u16)) {
        let r = self.brush_size.saturating_sub(1) as i16;
        if r == 0 {
            self.points.remove(&pos);
            return;
        }
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    self.points
                        .remove(&(pos.0.wrapping_add_signed(dx), pos.1.wrapping_add_signed(dy)));
                }
            }
        }
    }

    /// If symmetry mode is on, mirror `pos` across the vertical centre
    /// of the canvas and stamp there too. The mirror point is computed
    /// as (x_max - pos.x, pos.y); the centre column is NOT double-stamped
    /// (checked via `mx != pos.0`).
    pub(crate) fn place_symmetry(&mut self, pos: (u16, u16), color: Color) {
        if !self.symmetry_mode {
            return;
        }
        let x_max = self.canvas_area.width.saturating_sub(1);
        let mx = x_max.saturating_sub(pos.0);
        if mx != pos.0 {
            self.stamp_brush((mx, pos.1), color);
        }
    }

    /// Mirror erase — same logic as place_symmetry but erases.
    pub(crate) fn erase_line_symmetry(&mut self, pos: (u16, u16)) {
        if !self.symmetry_mode {
            return;
        }
        let x_max = self.canvas_area.width.saturating_sub(1);
        let mx = x_max.saturating_sub(pos.0);
        if mx != pos.0 {
            self.stamp_erase((mx, pos.1));
        }
    }
}

// ── Bresenham line ─────────────────────────────────────────────────
// Classic integer-only line algorithm. No floating point, no division.
// Returns every pixel coordinate along the line from `a` to `b`.
//
// The algorithm tracks an error term `err` that decides whether to
// step in x, y, or both. The input and output are (u16, u16), but
// internal math uses i32 to handle signed deltas.

impl DrawingApp {
    pub(crate) fn bresenham_points(&self, a: (u16, u16), b: (u16, u16)) -> Vec<(u16, u16)> {
        let mut pts = Vec::new();
        let (mut x0, mut y0) = (i32::from(a.0), i32::from(a.1));
        let (x1, y1) = (i32::from(b.0), i32::from(b.1));
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs(); // negative dy — canonical Bresenham
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy; // starts negative when |dx| < |dy|
        loop {
            pts.push((x0 as u16, y0 as u16));
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
        pts
    }
}

// ── Shape tools ────────────────────────────────────────────────────

impl DrawingApp {
    /// Outline rectangle from corner `a` to corner `b` (inclusive).
    pub(crate) fn stamp_rect(&mut self, a: (u16, u16), b: (u16, u16), color: Color) {
        let x1 = a.0.min(b.0);
        let x2 = a.0.max(b.0);
        let y1 = a.1.min(b.1);
        let y2 = a.1.max(b.1);
        for x in x1..=x2 {
            self.points.insert((x, y1), color);
            self.points.insert((x, y2), color);
        }
        for y in y1..=y2 {
            self.points.insert((x1, y), color);
            self.points.insert((x2, y), color);
        }
    }

    /// Filled rectangle — every pixel in the bounding box.
    pub(crate) fn stamp_filled_rect(&mut self, a: (u16, u16), b: (u16, u16), color: Color) {
        let x1 = a.0.min(b.0);
        let x2 = a.0.max(b.0);
        let y1 = a.1.min(b.1);
        let y2 = a.1.max(b.1);
        for x in x1..=x2 {
            for y in y1..=y2 {
                self.points.insert((x, y), color);
            }
        }
    }

    /// Midpoint circle algorithm (Bresenham circle). Only the outline.
    pub(crate) fn stamp_circle(&mut self, center: (u16, u16), radius: u16, color: Color) {
        let r = radius as i16;
        let cx = center.0 as i16;
        let cy = center.1 as i16;
        let mut x = r;
        let mut y = 0i16;
        let mut err = 1 - r;
        while x >= y {
            // Eight symmetric points per octant.
            let pts = [
                (cx + x, cy + y), (cx - x, cy + y),
                (cx + x, cy - y), (cx - x, cy - y),
                (cx + y, cy + x), (cx - y, cy + x),
                (cx + y, cy - x), (cx - y, cy - x),
            ];
            for (px, py) in pts {
                if px >= 0 && py >= 0 {
                    self.points.insert((px as u16, py as u16), color);
                }
            }
            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// Filled circle — scanline fill of the bounding box.
    /// For each row, computes the horizontal span via sqrt(radius² - dy²).
    pub(crate) fn stamp_filled_circle(&mut self, center: (u16, u16), radius: u16, color: Color) {
        let r = radius as i16;
        let cx = center.0 as i16;
        let cy = center.1 as i16;
        for dy in -r..=r {
            let dx = ((r * r - dy * dy) as f64).sqrt().round() as i16;
            for x in (cx - dx).max(0)..=(cx + dx).max(0) {
                let py = cy + dy;
                if py >= 0 {
                    self.points.insert((x as u16, py as u16), color);
                }
            }
        }
    }
}

// ── Gradient fill ──────────────────────────────────────────────────

impl DrawingApp {
    /// Fill the rectangle between `a` and `b` with a linear gradient
    /// from `ca` (top-left) to `cb` (bottom-right). Interpolation uses
    /// normalised (x/w + y/h) / 2 as the blend factor.
    ///
    /// The gradient is bilinear — it blends from ca at (x1,y1) to cb
    /// at (x2,y2), with intermediate colours computed via `lerp_color`.
    pub(crate) fn gradient_fill(&mut self, a: Position, b: Position, ca: Color, cb: Color) {
        let x1 = a.x.min(b.x);
        let x2 = a.x.max(b.x);
        let y1 = a.y.min(b.y);
        let y2 = a.y.max(b.y);
        let w = (x2 - x1).max(1) as f64;
        let h = (y2 - y1).max(1) as f64;
        for x in x1..=x2 {
            for y in y1..=y2 {
                let t = ((x - x1) as f64 / w + (y - y1) as f64 / h) / 2.0;
                let color = self.lerp_color(ca, cb, t);
                self.points.insert((x, y), color);
            }
        }
    }

    /// Linearly interpolate between two colours.
    /// t=0.0 → a, t=1.0 → b, clamped to [0,1].
    fn lerp_color(&self, a: Color, b: Color, t: f64) -> Color {
        let (r1, g1, b1) = color_to_rgb(a);
        let (r2, g2, b2) = color_to_rgb(b);
        let t = t.clamp(0.0, 1.0);
        Color::Rgb(
            (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8,
            (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8,
            (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8,
        )
    }
}

// ── Flood fill ─────────────────────────────────────────────────────

impl DrawingApp {
    /// 4-directional flood fill. Only replaces existing pixels (does NOT
    /// fill empty space). Uses a stack (DFS) with a visited set to avoid
    /// re-processing. Aborts if target colour == fill colour.
    ///
    /// Why stack-based DFS instead of queue-based BFS?
    ///   Both work; the stack uses a simple Vec as a LIFO, which is
    ///   cache-friendly and avoids the overhead of a VecDeque.
    pub(crate) fn flood_fill(&mut self, start: (u16, u16)) {
        let target = match self.points.get(&start) {
            Some(&c) => c,
            None => return,
        };
        let fill = self.palette.current();
        if target == fill {
            return;
        }
        let mut stack = vec![start];
        let mut visited = std::collections::BTreeMap::new();
        while let Some(pos) = stack.pop() {
            if visited.contains_key(&pos) {
                continue;
            }
            visited.insert(pos, ());
            if self.points.get(&pos) != Some(&target) {
                continue;
            }
            self.points.insert(pos, fill);
            for (dx, dy) in &[(0i16, 1i16), (0, -1), (1, 0), (-1, 0)] {
                stack.push((
                    pos.0.wrapping_add_signed(*dx),
                    pos.1.wrapping_add_signed(*dy),
                ));
            }
        }
    }
}

// ── Drag drawing / erasing ─────────────────────────────────────────
// When the user drags the mouse, we interpolate between the last known
// position and the current position using the Bresenham algorithm,
// stamping/erasing along the entire path.

impl DrawingApp {
    /// Stamp the current brush colour along a Bresenham line from
    /// `last_localition` to `end`. Also applies symmetry if active.
    /// If there's no previous position, just stamp at `end`.
    pub(crate) fn draw_line(&mut self, end: Position) {
        let color = self.draw_color();
        let Some(start) = self.last_localition else {
            self.stamp_brush((end.x, end.y), color);
            self.place_symmetry((end.x, end.y), color);
            self.last_localition = Some(end);
            return;
        };
        let (mut x0, mut y0) = (i32::from(start.x), i32::from(start.y));
        let (x1, y1) = (i32::from(end.x), i32::from(end.y));
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.stamp_brush((x0 as u16, y0 as u16), color);
            self.place_symmetry((x0 as u16, y0 as u16), color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
        self.last_localition = Some(end);
    }

    /// Same as draw_line but erases instead of stamping.
    pub(crate) fn erase_line(&mut self, end: Position) {
        let Some(start) = self.last_localition else {
            self.stamp_erase((end.x, end.y));
            self.erase_line_symmetry((end.x, end.y));
            self.last_localition = Some(end);
            return;
        };
        let (mut x0, mut y0) = (i32::from(start.x), i32::from(start.y));
        let (x1, y1) = (i32::from(end.x), i32::from(end.y));
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.stamp_erase((x0 as u16, y0 as u16));
            self.erase_line_symmetry((x0 as u16, y0 as u16));
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
        self.last_localition = Some(end);
    }
}

// ── Eyedropper ─────────────────────────────────────────────────────

impl DrawingApp {
    /// Pick the colour of the pixel at `pos` and select it in the palette.
    /// Only matches palette colours (not custom RGB colours).
    /// If the pixel colour isn't in the palette, nothing happens.
    pub(crate) fn pick_color_at(&mut self, pos: Position) {
        if let Some(&color) = self.points.get(&(pos.x, pos.y)) {
            let idx = self
                .palette
                .colors
                .iter()
                .position(|(c, _)| *c == color);
            if let Some(i) = idx {
                self.palette.select(i);
                self.custom_color_override = None;
                self.push_color_history(color);
            } else {
                // Colour isn't in the palette (e.g. loaded from an image).
                // Set it as the override instead of silently doing nothing.
                self.custom_color_override = Some(color);
                self.push_color_history(color);
            }
        }
    }
}

// ── Cellular automaton ────────────────────────────────────────────

impl DrawingApp {
    /// Seed a circular region with random live cells for Game of Life.
    pub(crate) fn seed_life_cells(&mut self, center: Position) {
        let radius = (self.brush_size * 5).max(5) as i16;
        let mut seed = self.color_gen_seed;
        let color = self.draw_color();
        for _ in 0..(radius as u16 * 15) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let dx = (seed >> 33) as i16 % (radius * 2 + 1) - radius;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let dy = (seed >> 33) as i16 % (radius * 2 + 1) - radius;
            if dx * dx + dy * dy <= radius * radius && !seed.is_multiple_of(4) {
                self.points.insert(
                    (center.x.wrapping_add_signed(dx), center.y.wrapping_add_signed(dy)),
                    color,
                );
            }
        }
        self.color_gen_seed = seed;
    }

    /// Advance one generation of Conway's Game of Life on the entire canvas.
    /// Live cell with 2 or 3 neighbours survives; dead cell with 3 neighbours
    /// is born. All other cells die.
    pub(crate) fn run_life_generation(&mut self) {
        let current = std::mem::take(&mut self.points);
        let mut neighbor_counts: std::collections::BTreeMap<(u16, u16), u8> =
            std::collections::BTreeMap::new();

        for &(x, y) in current.keys() {
            for dx in [-1i16, 0, 1] {
                for dy in [-1i16, 0, 1] {
                    if dx == 0 && dy == 0 { continue; }
                    *neighbor_counts
                        .entry((x.wrapping_add_signed(dx), y.wrapping_add_signed(dy)))
                        .or_insert(0) += 1;
                }
            }
        }

        let birth_color = self.draw_color();
        for (pos, count) in neighbor_counts {
            if current.contains_key(&pos) {
                if count == 2 || count == 3 {
                    let &color = current.get(&pos).unwrap_or(&birth_color);
                    self.points.insert(pos, color);
                }
            } else if count == 3 {
                self.points.insert(pos, birth_color);
            }
        }
    }
}

// ── Posterize ─────────────────────────────────────────────────────

impl DrawingApp {
    /// Reduce the canvas to the `n` most frequently used colours.
    /// Each pixel is mapped to the nearest of those `n` colours by
    /// Euclidean distance in RGB space.
    pub(crate) fn posterize(&mut self, n: usize) {
        if self.points.is_empty() { return; }

        let mut freq: std::collections::HashMap<Color, usize> =
            std::collections::HashMap::new();
        for &c in self.points.values() {
            *freq.entry(c).or_insert(0) += 1;
        }

        let mut sorted: Vec<(Color, usize)> = freq.into_iter().collect();
        sorted.sort_by_key(|k| std::cmp::Reverse(k.1));
        let top_n: Vec<Color> = sorted.into_iter().take(n).map(|(c, _)| c).collect();
        if top_n.is_empty() { return; }

        let snapshot: Vec<((u16, u16), Color)> = self.points.iter()
            .map(|(&pos, &c)| (pos, c))
            .collect();

        for (pos, color) in snapshot {
            let nearest = top_n.iter()
                .min_by_key(|target| {
                    let (r1, g1, b1) = color_to_rgb(color);
                    let (r2, g2, b2) = color_to_rgb(**target);
                    let dr = r1 as i32 - r2 as i32;
                    let dg = g1 as i32 - g2 as i32;
                    let db = b1 as i32 - b2 as i32;
                    dr * dr + dg * dg + db * db
                })
                .unwrap_or(&top_n[0]);
            self.points.insert(pos, *nearest);
        }
    }
}
