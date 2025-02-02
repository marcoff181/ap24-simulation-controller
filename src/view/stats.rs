use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::{self},
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::{network::node_kind::NodeKind, screen::Screen, utilities::theme::*, Network};

use super::packet_formatter::{message_table_row, packet_table_row};

pub fn render_stats(network: &Network, screen: &Screen, area: Rect, frame: &mut Frame) {
    let [r2, r3, r4] = Layout::horizontal([
        Constraint::Fill(3),
        Constraint::Fill(3),
        Constraint::Fill(3),
    ])
    .areas(area);

    let border_set2 = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.horizontal_down,
        bottom_right: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    let border_set3 = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        top_right: symbols::line::NORMAL.horizontal_down,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        bottom_right: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    let border_set4 = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        top_right: symbols::line::NORMAL.vertical_left,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };
    let (title3, title4) = match screen.kind {
        NodeKind::Drone { .. } => ("Packets Dropped", "Packets Shortcutted"),
        _ => ("Messages Sent", "Messages Received"),
    };

    let b2 = Block::new()
        .border_set(border_set2)
        .borders(Borders::ALL)
        .title("Packets Sent")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let b3 = Block::new()
        .border_set(border_set3)
        .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
        .title(title3)
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let b4 = Block::new()
        .border_set(border_set4)
        .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
        .title(title4)
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let n = network.get_node_from_id(screen.focus).unwrap();
    let pwidths = [
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(10),
    ];

    let mwidths = [
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(10),
    ];
    let pheader = Row::new(vec!["typ", "sid", "src", "dst", "about"]);
    let mheader = Row::new(vec!["typ", "←/→", "src", "dst", "sid", "about"]);

    match screen.kind {
        NodeKind::Drone { .. } => {
            let rows: Vec<Row<'_>> = n.sent.iter().map(|p| packet_table_row(p)).collect();
            let table = Table::new(rows, pwidths)
                .column_spacing(1)
                .header(pheader.clone())
                .block(b2);
            frame.render_widget(table, r2);

            let rows: Vec<Row<'_>> = n.dropped.iter().map(|p| packet_table_row(p)).collect();
            let table = Table::new(rows, pwidths)
                .column_spacing(1)
                .header(pheader.clone())
                .block(b3);
            frame.render_widget(table, r3);

            let rows: Vec<Row<'_>> = n.shortcutted.iter().map(|p| packet_table_row(p)).collect();
            let table = Table::new(rows, pwidths)
                .column_spacing(1)
                .header(pheader)
                .block(b4);
            frame.render_widget(table, r4);
        }
        _ => {
            let rows: Vec<Row<'_>> = n.sent.iter().map(|p| packet_table_row(p)).collect();
            let table = Table::new(rows, pwidths)
                .column_spacing(1)
                .style(Style::new().red())
                .header(pheader.clone())
                .block(b2);
            frame.render_widget(table, r2);

            let rows: Vec<Row<'_>> = n
                .msent
                .values()
                .rev()
                .map(|p| message_table_row(&p.0, p.1))
                .collect();
            let table = Table::new(rows, mwidths)
                .column_spacing(1)
                .header(mheader.clone())
                .block(b3);
            frame.render_widget(table, r3);

            let rows: Vec<Row<'_>> = n
                .mreceived
                .iter()
                .map(|p| message_table_row(p, true))
                .collect();
            let table = Table::new(rows, mwidths)
                .column_spacing(1)
                .header(mheader)
                .block(b4);
            frame.render_widget(table, r4);
        }
    }
}
