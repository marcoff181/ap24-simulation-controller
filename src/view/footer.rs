use super::keys::{
    DETAIL_KEYS_DRONE, DETAIL_KEYS_NOTDRONE, ERROR_KEYS, MAIN_KEYS, MAIN_KEYS_ADD_CONNECTION,
    MOVE_KEYS, PDR_KEYS,
};
use crate::{
    screen::{Screen, Window},
    utilities::theme::{BOTTOMPANEL_BG, HIGHLIGHT_COLOR, INVERTED_TEXT_COLOR, TEXT_COLOR},
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::{network::node_kind::NodeKind, Network};

pub fn render_footer(_network: &Network, screen: &Screen, area: Rect, buf: &mut Buffer) {
    let keys: &[(&str, &str)] = match screen.window {
        Window::Main => &MAIN_KEYS,
        Window::Move => &MOVE_KEYS,
        Window::AddConnection { origin: _ } => &MAIN_KEYS_ADD_CONNECTION,
        Window::ChangePdr { pdr: _ } => &PDR_KEYS,
        Window::Detail { tab: _ } => match screen.kind {
            NodeKind::Drone { pdr: _, crashed: _ } => &DETAIL_KEYS_DRONE,
            NodeKind::Client | NodeKind::Server => &DETAIL_KEYS_NOTDRONE,
        },
        Window::Error { message: _ } => &ERROR_KEYS,
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
