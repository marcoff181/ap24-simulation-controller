use crate::{screen::Screen, utilities::theme::*};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, HighlightSpacing, List, ListDirection, ListState, StatefulWidget},
};

use crate::Network;

pub fn render_list(
    network: &Network,
    screen: &Screen,
    node_list_state: &mut ListState,
    area: Rect,
    buf: &mut Buffer,
) {
    let _ = screen;
    let left_block = Block::new()
        .borders(Borders::TOP | Borders::LEFT)
        .title("Nodes")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let items = network
        .nodes
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>();
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

    StatefulWidget::render(list, area, buf, node_list_state);
}
