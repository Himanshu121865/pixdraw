
use ratatui::{
    layout::Position,
    style::Color,
};

use crate::app::DrawingApp;

impl DrawingApp {
            pub(crate) fn copy_selection(&mut self) {
        let Some(start) = self.selection_start else { return };
        let Some(end) = self.selection_end else { return };
        self.selection_buffer.clear();
        let x1 = start.x.min(end.x);
        let x2 = start.x.max(end.x);
        let y1 = start.y.min(end.y);
        let y2 = start.y.max(end.y);
        for x in x1..=x2 {
            for y in y1..=y2 {
                if let Some(&color) = self.points.get(&(x, y)) {
                    self.selection_buffer.push((x - x1, y - y1, color));
                }
            }
        }
    }

        pub(crate) fn cut_selection(&mut self) {
        self.copy_selection();
        let Some(start) = self.selection_start else { return };
        let Some(end) = self.selection_end else { return };
        self.push_history();
        let x1 = start.x.min(end.x);
        let x2 = start.x.max(end.x);
        let y1 = start.y.min(end.y);
        let y2 = start.y.max(end.y);
        for x in x1..=x2 {
            for y in y1..=y2 {
                self.points.remove(&(x, y));
            }
        }
    }

            pub(crate) fn paste_selection(&mut self, at: Position) {
        self.push_history();
        let to_insert: Vec<(u16, u16, Color)> = self.selection_buffer.iter()
            .map(|(dx, dy, color)| (at.x + dx, at.y + dy, *color))
            .collect();
        for (x, y, color) in to_insert {
            self.points.insert((x, y), color);
        }
    }

                pub(crate) fn nudge_selection(&mut self, dx: i16, dy: i16) {
        let Some(start) = self.selection_start else { return };
        let Some(end) = self.selection_end else { return };
        self.push_history();
        let x1 = start.x.min(end.x);
        let x2 = start.x.max(end.x);
        let y1 = start.y.min(end.y);
        let y2 = start.y.max(end.y);

                let mut selected: Vec<(u16, u16, Color)> = Vec::new();
        for x in x1..=x2 {
            for y in y1..=y2 {
                if let Some(&c) = self.points.get(&(x, y)) {
                    selected.push((x, y, c));
                }
            }
        }
                for (x, y, _) in &selected {
            self.points.remove(&(*x, *y));
        }
                for (x, y, c) in selected {
            let nx = x.wrapping_add_signed(dx);
            let ny = y.wrapping_add_signed(dy);
            self.points.insert((nx, ny), c);
        }
                self.selection_start = Some(Position::new(
            start.x.wrapping_add_signed(dx),
            start.y.wrapping_add_signed(dy),
        ));
        self.selection_end = Some(Position::new(
            end.x.wrapping_add_signed(dx),
            end.y.wrapping_add_signed(dy),
        ));
    }
}
