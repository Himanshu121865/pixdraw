
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use super::col::*;


struct HelpCat(&'static str, &'static [&'static str]);

const HELP_DATA: &[HelpCat] = &[
    HelpCat(" Drawing", &[
        "  left-click         draw pixel / anchor point",
        "  left-drag          draw line",
        "  Shift+click/drag   constrain (45°/square)",
        "  right-click        context menu",
        "  middle-click       clear canvas",
        "  e                  toggle eraser mode",
        "  i                  eyedropper (pick colour)",
        "  f                  flood fill",
        "  m                  toggle mirror",
    ]),
    HelpCat(" Shapes", &[
        "  r                  rectangle (drag to shape)",
        "  R                  filled rectangle (drag)",
        "  o                  circle outline (drag)",
        "  O                  filled circle (drag)",
        "  l                  continuous line (click segments)",
        "  p                  spray can mode",
        "  b                  rainbow brush toggle",
    ]),
    HelpCat(" Selection", &[
        "  s                  toggle select mode",
        "  click+drag         select region",
        "  ^C                 copy selection",
        "  ^X                 cut selection",
        "  ^V                 paste at cursor",
        "  arrows             nudge selection",
    ]),
    HelpCat(" Colours", &[
        "  ^N                 open colour picker",
        "  click palette bar  pick colour (○ ● ◆ ◇)",
        "  Space              next colour",
        "  scroll up/down     previous/next colour",
        "  1-9                select palette colour",
        "  u                  generate 3 custom colours",
        "  0                  cycle custom colours",
        "  G                  gradient fill (2-click)",
    ]),
    HelpCat(" Brush / View", &[
        "  [ / -              shrink brush",
        "  ] / + / =          grow brush",
        "  g                  toggle grid overlay",
    ]),
    HelpCat(" Tab", &[
        "  ^T                 new tab",
        "  ^W                 close tab",
        "  Tab                next tab",
        "  Shift+Tab          previous tab",
        "  F2                 rename current tab",
    ]),
    HelpCat(" File", &[
        "  ^S                 save (.txt)",
        "  ^O                 open (.txt)",
        "  ^E                 export PNG",
        "  ^R                 resize canvas",
        "  ^Z / ^Y            undo / redo",
    ]),
    HelpCat(" Effects", &[
        "  L                  toggle cellular automaton",
        "  left-click         seed random cells",
        "  auto-advances      ~6 gen/s in life mode",
        "  right-click        manual step",
        "  P                  posterize (8 colour quantize)",
        "  ^P                 quick export PNG",
    ]),
    HelpCat(" Other", &[
        "  t                  text tool",
        "  c                  clear canvas",
        "  q                  quit",
        "  ? / /              toggle this help",
        "  9 / 0              expand / collapse all categories",
        "  Esc                close popup / cancel",
    ]),
];

pub fn render_help_popup(
    frame: &mut Frame<'_>,
    screen: Rect,
    expanded: &[bool; 9],
    selected: usize,
    scroll: u16,
    search_buffer: &str,
    search_active: bool,
) {
    let search_bar = format!(" {} Search: {}", if search_active { "▶" } else { " " }, search_buffer);

        let mut lines = Vec::new();
    if search_buffer.is_empty() {
        for (i, cat) in HELP_DATA.iter().enumerate() {
            let is_selected = i == selected;
            let cursor = if is_selected { "▸" } else { " " };
            let marker = if expanded[i] { "▼" } else { "▶" };
            let cat_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::White).bold()
            } else {
                Style::default().fg(accent()).bold()
            };
            lines.push(Line::from(Span::styled(
                format!(" {} {} {}", cursor, marker, cat.0),
                cat_style,
            )));
            if expanded[i] {
                for entry in cat.1 {
                    lines.push(Line::from(Span::styled(*entry, Style::default().fg(text()))));
                }
            }
        }
    } else {
        let q = search_buffer.to_lowercase();
        for cat in HELP_DATA.iter() {
            let matched: Vec<&&str> = cat.1.iter().filter(|e| e.to_lowercase().contains(&q)).collect();
            if matched.is_empty() { continue; }
            lines.push(Line::from(Span::styled(
                format!("    {}", cat.0),
                Style::default().fg(accent()).bold(),
            )));
            for entry in matched {
                lines.push(Line::from(Span::styled(*entry, Style::default().fg(text()))));
            }
        }
    }

    let width = 58u16.min(screen.width.saturating_sub(4));
    let max_height = screen.height.saturating_sub(2);
    let height = ((lines.len() as u16).saturating_add(3)).min(max_height);
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + (screen.height - height) / 2;
    let area = Rect::new(x, y, width, height);

    let bottom_hint = if search_active {
        "  type to filter  Esc to browse  Enter commit"
    } else {
        "  j/k navigate  Enter toggle  9 expand  0 collapse  Esc close"
    };

    let block = Block::bordered()
        .title(" Keybindings ")
        .border_style(Style::default().fg(border()))
        .title_bottom(Line::from(Span::styled(bottom_hint, Style::default().fg(subtle()))));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

        let search_style = if search_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(subtle())
    };
    let max_search_len = inner.width.saturating_sub(2) as usize;
    let display_search: String = search_bar.chars().take(max_search_len).collect();
    for (ci, ch) in display_search.chars().enumerate() {
        let cx = inner.x + ci as u16;
        if cx >= inner.x + inner.width { break; }
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, inner.y)) {
            cell.set_char(ch).set_style(search_style);
        }
    }
    for cx in (inner.x + display_search.len() as u16)..inner.x.saturating_add(inner.width) {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, inner.y)) {
            cell.set_char(' ').set_style(search_style);
        }
    }
    if search_active {
        let cursor_x = inner.x + display_search.len() as u16;
        if cursor_x < inner.x + inner.width
            && let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cursor_x, inner.y)) {
                cell.set_char('▌').set_style(Style::default().fg(Color::Green));
            }
    }

    let display_lines: Vec<Line<'_>> = lines.iter()
        .skip(scroll as usize)
        .take(inner.height.saturating_sub(1) as usize)
        .cloned()
        .collect();

    let content_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height.saturating_sub(1));
    frame.render_widget(Paragraph::new(display_lines), content_area);
}
