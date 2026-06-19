
use ratatui::style::Color;

pub const PALETTE: &[(Color, &str)] = &[
    (Color::Rgb(0xFF, 0xFF, 0xFF), "White"),
    (Color::Rgb(0xC2, 0xC3, 0xC7), "Light Gray"),
    (Color::Rgb(0x5F, 0x57, 0x4F), "Gray"),
    (Color::Rgb(0x00, 0x00, 0x00), "Black"),
    (Color::Rgb(0xFF, 0x00, 0x4D), "Red"),
    (Color::Rgb(0xFF, 0xA3, 0x00), "Orange"),
    (Color::Rgb(0xFF, 0xEC, 0x27), "Yellow"),
    (Color::Rgb(0x00, 0xE4, 0x36), "Green"),
    (Color::Rgb(0x00, 0x5A, 0x4A), "Dark Green"),
    (Color::Rgb(0x29, 0xAD, 0xFF), "Blue"),
    (Color::Rgb(0x2B, 0x3E, 0x5B), "Dark Blue"),
    (Color::Rgb(0xFF, 0x77, 0xA8), "Pink"),
    (Color::Rgb(0xAB, 0x52, 0x36), "Brown"),
    (Color::Rgb(0xFF, 0xCC, 0xAA), "Peach"),
];

pub struct Palette {
    pub colors: Vec<(Color, String)>,
    pub index: usize,
}

impl Palette {
    pub fn from_config(config: &crate::config::Palette) -> Self {
        let colors: Vec<(Color, String)> = config.colors.iter()
            .zip(config.names.iter())
            .map(|(c, n)| (*c, n.clone()))
            .collect();
        Self { colors, index: 0 }
    }

    pub fn current(&self) -> Color {
        self.colors[self.index].0
    }

    pub fn name(&self) -> &str {
        &self.colors[self.index].1
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.colors.len();
    }

    pub fn prev(&mut self) {
        self.index = if self.index == 0 {
            self.colors.len() - 1
        } else {
            self.index - 1
        };
    }

    pub fn select(&mut self, n: usize) {
        if n < self.colors.len() {
            self.index = n;
        }
    }
}
