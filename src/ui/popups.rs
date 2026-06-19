
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Widget},
};

use crate::app::DrawingApp;
use super::col::*;

pub fn render_file_browser(app: &mut DrawingApp, frame: &mut Frame<'_>, screen: Rect) {
    let width = 50u16.min(screen.width.saturating_sub(4));
    let height = 26u16.min(screen.height.saturating_sub(4));
    let x = screen.x + (screen.width - width) / 2;
    let y = screen.y + (screen.height - height) / 2;
    let area = Rect::new(x, y, width, height);
    app.file_browser_area = area;

    let title = match app.file_browser.mode {
        crate::file_browser::FileBrowserMode::Save => "Save",
        crate::file_browser::FileBrowserMode::ExportPng => "Export PNG",
        crate::file_browser::FileBrowserMode::Load => "Open",
    };

    let block = Block::bordered()
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(border()));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    block.render(area, frame.buffer_mut());

        let path_str = app.file_browser.current_path.to_string_lossy();
    let max_path_len = inner.width as usize;
    let display_path = if path_str.len() > max_path_len {
        format!("…{}", &path_str[path_str.len().saturating_sub(max_path_len - 1)..])
    } else {
        path_str.to_string()
    };
    Paragraph::new(Line::from(Span::styled(
        display_path,
        Style::default().fg(accent()),
    )))
    .render(Rect::new(inner.x, inner.y, inner.width, 1), frame.buffer_mut());

        for sx in inner.x..inner.x.saturating_add(inner.width) {
        if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(sx, inner.y + 1)) {
            cell.set_char('─').set_style(Style::default().fg(dim()));
        }
    }

        let list_y = inner.y + 2;
    let list_height = inner.height.saturating_sub(5);
    let entries: Vec<(String, bool)> = app
        .file_browser
        .entries
        .iter()
        .enumerate()
        .skip(app.file_browser.scroll_offset)
        .take(list_height as usize)
        .map(|(i, e)| {
            let icon = if e.is_dir {
                if e.name == ".." { "↑" } else { "▸" }
            } else {
                " "
            };
            let marker = if i == app.file_browser.selected { "▶" } else { " " };
            (format!("{} {} {}", marker, icon, e.name), e.is_dir)
        })
        .collect();

    for (j, (line, is_dir)) in entries.iter().enumerate() {
        let row_y = list_y + j as u16;
        if row_y >= inner.y + inner.height { break; }
        let is_selected = j + app.file_browser.scroll_offset == app.file_browser.selected;
        let fg = if is_selected { Color::Black } else if *is_dir { accent() } else { text() };
        let bg = if is_selected { Color::White } else { Color::Reset };
        let line_chars: Vec<char> = line.chars().collect();

        for (ci, &ch) in line_chars.iter().enumerate() {
            let cx = inner.x + ci as u16;
            if cx >= inner.x + inner.width { break; }
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, row_y)) {
                cell.set_char(ch).set_style(Style::default().fg(fg).bg(bg));
            }
        }

                for cx in (inner.x + line_chars.len() as u16)..inner.x.saturating_add(inner.width) {
            if let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cx, row_y)) {
                cell.set_char(' ').set_style(Style::default().bg(bg));
            }
        }
    }

        let footer_y = inner.y + inner.height - 2;
    let is_save_like = app.file_browser.mode == crate::file_browser::FileBrowserMode::Save
        || app.file_browser.mode == crate::file_browser::FileBrowserMode::ExportPng;
    if is_save_like {
        let mode_hint = if app.file_browser.filename_input_active {
            " [type]"
        } else {
            " [i=type]"
        };
        let input_label = Span::styled(
            format!("File{}{}", mode_hint, app.file_browser.filename_input),
            Style::default().fg(Color::Green),
        );
        Paragraph::new(Line::from(vec![input_label]))
            .render(Rect::new(inner.x, footer_y, inner.width, 1), frame.buffer_mut());
        if app.file_browser.filename_input_active {
            let cursor_x = inner.x + 6 + mode_hint.len() as u16 + app.file_browser.filename_input.len() as u16;
            if cursor_x < inner.x + inner.width
                && let Some(cell) = frame.buffer_mut().cell_mut(Position::new(cursor_x, footer_y)) {
                    cell.set_char('▌').set_style(Style::default().fg(Color::Green));
                }
        }
    } else {
        Paragraph::new(Line::from(Span::styled(
            " ↑↓ navigate  Enter open  Tab parent  Esc cancel",
            Style::default().fg(dim()),
        )))
        .render(Rect::new(inner.x, footer_y, inner.width, 1), frame.buffer_mut());
    }

        Paragraph::new(Line::from(Span::styled(
        format!(" ↑↓ navigate  Enter select  Tab parent  Esc cancel    [{} entries]",
            app.file_browser.entries.len()),
        Style::default().fg(dim()),
    )))
    .render(Rect::new(inner.x, inner.y + 1, inner.width, 1), frame.buffer_mut());
}

