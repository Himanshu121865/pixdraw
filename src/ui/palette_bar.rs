// ── ui/palette_bar.rs ───────────────────────────────────────────────
// Single-row palette bar (footer) with colour swatches, custom colour
// slots, and three colour-related popups: picker, selector, and input.
//
// The palette bar renders at the very bottom of the screen and shows:
//   "Palette:" label
//   14 colour swatches (○ for unselected, ● for current)
//   Separator ┆
//   Custom colour slots (C1◇ C2◆ etc.)
//
// All three colour popups (picker, selector, input) are rendered here
// because they're closely related to colour selection logic.

use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::app::DrawingApp;
use super::col::*;

/// Render the palette bar at the bottom footer area.
/// Each palette colour is a single character: ○ (not selected) or ● (selected).
/// Custom colours show as C1◇ (not active) or C1◆ (active).
pub fn render_palette_bar(
    frame: &mut Frame<'_>,
    area: Rect,
    current: Color,
    colors: &[(Color, &str)],
    custom_colors: &[Color],
    custom_override: Option<Color>,
) {
    let row = area.y;

    // Clear the row first.
    for x in area.x..area.x.saturating_add(area.width) {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, row)) {
            cell.set_char(' ').set_style(Style::default().bg(bg()));
        }
    }

    let mut x = area.x + 1;
    let max_x = area.x.saturating_add(area.width).saturating_sub(2);

    if x < max_x {
        let label = "Palette:";
        for (ci, ch) in label.chars().enumerate() {
            let px = x + ci as u16;
            if px >= max_x { break; }
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(px, row)) {
                cell.set_char(ch).set_style(Style::default().fg(subtle()));
            }
        }
        x += label.len() as u16 + 1;
    }

    for (color, _) in colors {
        if x > max_x { break; }
        let ch = if *color == current { '●' } else { '○' };
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, row)) {
            cell.set_char(ch).set_style(Style::default().fg(*color));
        }
        x += 1;
    }

    // Separator between palette and custom colours.
    if !custom_colors.is_empty() && x < max_x {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, row)) {
            cell.set_char('┆').set_style(Style::default().fg(subtle()));
        }
        x += 1;
    }

    // Custom colour slots.
    for (slot, c) in custom_colors.iter().enumerate() {
        if x >= max_x { break; }
        let label = format!("C{}", slot + 1);
        for (ci, ch) in label.chars().enumerate() {
            let px = x + ci as u16;
            if px > max_x { break; }
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(px, row)) {
                cell.set_char(ch).set_style(Style::default().fg(subtle()));
            }
        }
        x += label.len() as u16;

        if x > max_x { break; }
        let is_active = custom_override == Some(*c);
        let ch = if is_active { '◆' } else { '◇' };
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(x, row)) {
            cell.set_char(ch).set_style(Style::default().fg(*c));
        }
        x += 1;
    }
}

/// Render the colour picker overlays on the canvas edge.
/// Shows all palette colours and custom colours in a vertical list.
/// The picker is triggered by ^Tab or clicking the colour swatch.
pub fn render_color_picker(app: &mut DrawingApp, frame: &mut Frame<'_>, canvas: Rect) {
    let custom_count = app.custom_colors.len() as u16;
    let total = app.palette.colors.len() as u16 + custom_count;
    let width = 24u16.min(canvas.width.saturating_sub(2));
    let height = total + 2;
    let x = canvas.x;
    let y = canvas.y + canvas.height - height;
    let area = Rect::new(x, y, width, height);

    // Fill background.
    for yy in area.y..area.y.saturating_add(area.height) {
        for xx in area.x..area.x.saturating_add(area.width) {
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(xx, yy)) {
                cell.set_char(' ').set_style(Style::default().bg(bg()));
            }
        }
    }

    let block = Block::bordered()
        .title(" Colors ")
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());
    app.color_picker_area = area;

    let mut lines: Vec<Line> = app
        .palette
        .colors
        .iter()
        .enumerate()
        .map(|(i, (c, name))| {
            let marker = if i == app.palette.index { "▸" } else { " " };
            Line::from(vec![
                Span::styled(marker, Style::default().fg(accent())),
                Span::styled(format!(" {:<12}", name), Style::default().fg(text())),
                Span::styled("●", Style::default().fg(*c)),
            ])
        })
        .collect();

    for (i, c) in app.custom_colors.iter().enumerate() {
        let marker = if app.custom_color_override == Some(*c) { "▸" } else { " " };
        let (r, g, b) = match c {
            Color::Rgb(r, g, b) => (*r, *g, *b),
            _ => (0, 0, 0),
        };
        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(accent())),
            Span::styled(format!("C{:<11}", i + 1), Style::default().fg(subtle())),
            Span::styled("●", Style::default().fg(*c)),
            Span::styled(format!(" {:>3},{:>3},{:>3}", r, g, b), Style::default().fg(*c)),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the colour selector popup for custom colours.
/// Triggered by pressing `u` to generate 3 random colours.
/// Shows the three generated colours with their RGB values.
pub fn render_color_selector(
    frame: &mut Frame<'_>,
    area: Rect,
    custom_colors: &[Color],
    selected_idx: usize,
) {
    let count = custom_colors.len() as u16;
    if count == 0 { return; }
    let width = 34u16.min(area.width.saturating_sub(4));
    let height = count + 4;
    let x = area.x + (area.width - width) / 2;
    let y = area.y.saturating_add(2);
    let popup = Rect::new(x, y, width, height);

    let block = Block::bordered()
        .title(" Pick a Colour ")
        .border_style(Style::default().fg(border()));
    let inner = block.inner(popup);
    block.render(popup, frame.buffer_mut());

    let mut lines: Vec<Line> = Vec::new();
    for (i, c) in custom_colors.iter().enumerate() {
        let is_selected = i == selected_idx;
        let prefix = if is_selected { "▸" } else { " " };
        let (r, g, b) = match c {
            Color::Rgb(r, g, b) => (*r, *g, *b),
            _ => (0, 0, 0),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", prefix), Style::default().fg(if is_selected { accent() } else { dim() })),
            Span::styled(format!(" C{} ", i + 1), Style::default().fg(dim())),
            Span::styled("██", Style::default().fg(*c).bg(*c)),
            Span::styled(format!("  rgb({:>3},{:>3},{:>3})", r, g, b), Style::default().fg(*c)),
        ]));
    }

    // "Custom RGB..." option — always appears after the generated colours.
    let is_custom_selected = selected_idx == custom_colors.len();
    let custom_prefix = if is_custom_selected { "▸" } else { " " };
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", custom_prefix), Style::default().fg(if is_custom_selected { accent() } else { dim() })),
        Span::styled(" Custom RGB...", Style::default().fg(if is_custom_selected { Color::White } else { Color::Gray })),
    ]));

    lines.push(Line::from(Span::styled(
        "  ↑↓/jk navigate  Enter select  Esc",
        Style::default().fg(dim()),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render the custom RGB colour input dialog.
/// Users type "R G B" or "R,G,B" values and see a live preview of the colour.
pub fn render_color_input(frame: &mut Frame<'_>, screen: Rect, buffer: &str) {
    let width = 34u16.min(screen.width.saturating_sub(4));
    let height = 6;
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + screen.height.saturating_sub(9);
    let area = Rect::new(x, y, width, height);

    let block = Block::bordered()
        .title(" Custom RGB ")
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

    // Live preview of the colour being typed.
    let preview_color = DrawingApp::parse_rgb_buffer(buffer);
    let line1 = if let Some(c) = preview_color {
        let (r, g, b) = match c {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (0, 0, 0),
        };
        Line::from(vec![
            Span::styled(" ██████ ", Style::default().fg(c).bg(c)),
            Span::styled(format!("  rgb({:>3},{:>3},{:>3})", r, g, b), Style::default().fg(c)),
            Span::styled(format!("  #{:02X}{:02X}{:02X}", r, g, b), Style::default().fg(c)),
        ])
    } else {
        Line::from(Span::styled(" type R G B or R,G,B", Style::default().fg(dim())))
    };

    let display = format!(" {}", buffer);
    let line2 = Line::from(vec![
        Span::styled(&display, Style::default().fg(Color::Green)),
        Span::styled("█", Style::default().fg(Color::Green)),
    ]);

    frame.render_widget(Paragraph::new(vec![line1, line2]), inner);
}
