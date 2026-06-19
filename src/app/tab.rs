
use std::collections::BTreeMap;
use ratatui::{layout::Position, style::Color};
use crate::app::ShapeKind;

pub struct TabData {
                pub points: BTreeMap<(u16, u16), Color>,

                pub text_entries: Vec<(Position, String)>,

                            pub history: Vec<BTreeMap<(u16, u16), Color>>,

        pub redo_stack: Vec<BTreeMap<(u16, u16), Color>>,

        pub text_mode: bool,
        pub text_cursor: Option<Position>,
        pub text_buffer: String,

    pub select_mode: bool,
        pub selecting: bool,
    pub selection_start: Option<Position>,
    pub selection_end: Option<Position>,
        pub selection_buffer: Vec<(u16, u16, Color)>,

            pub shape_mode: Option<ShapeKind>,
        pub shape_anchor: Option<Position>,

    pub line_mode: bool,
    pub line_anchor: Option<Position>,

    pub gradient_mode: bool,
    pub gradient_anchor: Option<Position>,

                pub last_localition: Option<Position>,

        pub name: String,
}

impl TabData {
    pub fn new(name: String) -> Self {
        Self {
            points: BTreeMap::new(),
            text_entries: Vec::new(),
            history: Vec::new(),
            redo_stack: Vec::new(),
            text_mode: false,
            text_cursor: None,
            text_buffer: String::new(),
            select_mode: false,
            selecting: false,
            selection_start: None,
            selection_end: None,
            selection_buffer: Vec::new(),
            shape_mode: None,
            shape_anchor: None,
            line_mode: false,
            line_anchor: None,
            gradient_mode: false,
            gradient_anchor: None,
            last_localition: None,
            name,
        }
    }
}
