
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{utilities::theme::*, Model};


pub fn render_stats(model: &Model, area: Rect, buf: &mut Buffer) {
    let [r1, r2, r3] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    //let collapsed_top_and_left_border_set = symbols::border::Set {
    //    top_left: symbols::line::NORMAL.vertical_right,
    //    top_right: symbols::line::NORMAL.vertical_left,
    //    bottom_left: symbols::line::NORMAL.horizontal_up,
    //    ..symbols::border::PLAIN
    //};
    //
    //// let bottom_right_block =
    //Block::new()
    //    .border_set(collapsed_top_and_left_border_set)
    //    .borders(Borders::ALL)
    //    .title("Stats")
    //    .bg(BG_COLOR)
    //    .fg(TEXT_COLOR)
    //    .render(area, buf);

    let border_set1 = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.horizontal_down,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        bottom_right: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    let border_set2 = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        top_right: symbols::line::NORMAL.horizontal_down,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        bottom_right: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    let border_set3 = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        top_right: symbols::line::NORMAL.vertical_left,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    let b1 = Block::new()
        .border_set(border_set1)
        .borders(Borders::all())
        .title("Stats")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let b2 = Block::new()
        .border_set(border_set2)
        .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
        .title("Sent")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let b3 = Block::new()
        .border_set(border_set3)
        .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
        .title("Received")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    // Stats
    let desc_text = Text::styled(format!("Pdr:{}", 4), Style::new());

    // Packets Sent

    Paragraph::new(desc_text).block(b1).render(r1, buf);
    b2.render(r2, buf);
    b3.render(r3, buf);
}
