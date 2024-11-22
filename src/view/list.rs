use crate::utilities::theme::*;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, HighlightSpacing, List, ListDirection, StatefulWidget},
};

use crate::Model;

pub fn render_list(model: &mut Model, area: Rect, buf: &mut Buffer) {
    let left_block = Block::new()
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .title("Nodes")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let items = model
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

    StatefulWidget::render(list, area, buf, &mut model.node_list_state);
}
