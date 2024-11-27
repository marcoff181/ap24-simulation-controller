use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::Text,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListDirection, Paragraph, Row, StatefulWidget,
        Table, Widget,
    },
};

use crate::{utilities::theme::*, Model};

use super::packet_formatter::format_packet;

pub fn render_stats(model: &Model, area: Rect, buf: &mut Buffer) {
    let [r1, r2, r3] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(3),
        Constraint::Fill(3),
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

    let widths = [
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Min(10),
    ];
    let header = Row::new(vec!["", "sid", "src", "dest", "about"]);

    if let Some(n) = model.get_selected_node() {
        let rows: Vec<Row<'_>> = n.sent.iter().map(|p| format_packet(p)).collect();
        let table = Table::new(rows, widths)
            .column_spacing(1)
            .style(Style::new().red())
            .header(header.clone())
            .block(b2);
        Widget::render(table, r2, buf);
    } else {
        b2.render(r2, buf);
    }
    // Packets received
    if let Some(n) = model.get_selected_node() {
        let rows: Vec<Row<'_>> = n.received.iter().map(|p| format_packet(p)).collect();
        let table = Table::new(rows, widths)
            .column_spacing(1)
            .style(Style::new().red())
            .header(header)
            .block(b3);
        Widget::render(table, r3, buf);
    } else {
        b3.render(r3, buf);
    }

    Paragraph::new(desc_text).block(b1).render(r1, buf);
}
