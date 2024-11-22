use crate::utilities::theme::*;
use super::keys::*;


use ratatui::{
    buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::{self, Marker}, text::{Line, Span}, widgets::{canvas::Canvas, Block, Borders, HighlightSpacing, List, ListDirection, Padding, StatefulWidget, Widget}
};

use crate::{model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen}, utilities::theme::*, Model};

pub fn render_footer(model:&Model, area: Rect, buf: &mut Buffer) {
    

    let move_keys = [("↑/↓/→/←", "Move"), ("Enter", "Ok"), ("q", "Quit")];

    let keys: &[(&str, &str)] = match screen {
        Screen::Main => match get_selected_kind() {
            Some(NodeKind::Drone) => &MAIN_KEYS_OVER_DRONE,
            _ => &MAIN_KEYS,
        },
        Screen::Start => &START_KEYS,
        Screen::Move => &move_keys,
        Screen::AddConnection { origin: _ } => &MAIN_KEYS_ADD_CONNECTION,
        Screen::AddNode => &MAIN_KEYS_ADD_NODE,
    };

    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(key, desc)| {
            let key = Span::styled(
                format!(" {key} "),
                Style::new().fg(INVERTED_TEXT_COLOR).bg(HIGHLIGHT_COLOR),
            );
            let desc = Span::styled(
                format!(" {desc} "),
                Style::new().fg(TEXT_COLOR).bg(BOTTOMPANEL_BG),
            );
            [key, desc]
        })
        .collect();

    Line::from(spans)
        .centered()
        .style((Color::Yellow, BOTTOMPANEL_BG))
        .render(area, buf);
}