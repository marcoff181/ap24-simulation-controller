use crate::utilities::theme::*;
use std::collections::HashSet;

use ratatui::{
    buffer::Buffer, layout::Rect, style::{Color, Style, Stylize}, symbols::{self, Marker}, text::Line, widgets::{canvas::Canvas, Block, Borders, HighlightSpacing, List, ListDirection, Padding, StatefulWidget, Widget}
};

use crate::{model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen}, utilities::theme::*, Model};

pub fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
    let left_block = Block::new()
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .title("Nodes")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let items = self
        .nodes
        .iter()
        .map(|x| x.repr.as_str())
        .collect::<Vec<&str>>();
    //let items = ["Drone  #12321","Drone  #12321","Drone  #12321","Drone  #12321", "Client #22343", "Server #32342"];
    let list = List::new(items)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().bold().bg(HIGHLIGHT_COLOR))
        .highlight_symbol("»")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom)
        .block(left_block)
        .highlight_spacing(HighlightSpacing::Always);

    StatefulWidget::render(list, area, buf, &mut self.node_list_state);
}