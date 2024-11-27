use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::{self, line::NORMAL},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListDirection, Paragraph, Row,
        StatefulWidget, Table, Widget,
    },
    Frame,
};

use crate::{model::node_kind::NodeKind, utilities::theme::*, Model};

use super::packet_formatter::format_packet;

pub fn render_stats(model: &Model, area: Rect, frame: &mut Frame) {
    let [r1, r2, r3] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(3),
        Constraint::Fill(3),
    ])
    .areas(area);

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
        .title("Dropped")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    // Stats
    if let Some(n) = model.get_selected_node() {
        let mut spans: Vec<Line> = vec![];
        match n.kind {
            NodeKind::Client | NodeKind::Server => {}
            NodeKind::Drone { pdr, crashed } => {
                spans.push(Line::from(format!("Pdr:{}", pdr)));
                spans.push(Line::from(format!("Crashed:{}", crashed)));
            }
        }
        spans.push(Line::from(format!("adj:{:?}", n.adj)));
        Paragraph::new(Text::from(spans))
            .block(b1)
            .render(r1, frame.buffer_mut());
    }
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
        frame.render_widget(table, r2);
    } else {
        b2.render(r2, frame.buffer_mut());
    }
    // Packets dropped
    if let Some(n) = model.get_selected_node() {
        let rows: Vec<Row<'_>> = n.dropped.iter().map(|p| format_packet(p)).collect();
        let table = Table::new(rows, widths)
            .column_spacing(1)
            .style(Style::new().red())
            .header(header)
            .block(b3);
        frame.render_widget(table, r3);
    } else {
        b3.render(r3, frame.buffer_mut());
    }
}
