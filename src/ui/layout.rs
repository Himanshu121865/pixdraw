
use ratatui::layout::Rect;

pub fn layout(area: Rect) -> (Rect, Rect, Rect, Rect, Rect) {
                        let header = Rect::new(area.x, area.y, area.width, 3);
    let body = Rect::new(
        area.x,
        area.y.saturating_add(3),
        area.width,
        area.height.saturating_sub(5),
    );
    let status_y = body.y.saturating_add(body.height);
    let status = Rect::new(area.x, status_y, area.width, 1);
    let footer = Rect::new(area.x, status_y.saturating_add(1), area.width, 1);
    let canvas = Rect::new(body.x, body.y, body.width, body.height);
    (header, body, footer, status, canvas)
}
