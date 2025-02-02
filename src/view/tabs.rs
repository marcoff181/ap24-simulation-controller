use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Row, Table, TableState, Tabs, Widget},
    Frame,
};

use crate::{
    network::{node_kind::NodeKind, Network},
    screen::Screen,
    utilities::theme::TEXT_COLOR,
};

use super::packet_formatter::{message_table_row, packet_table_row};

pub fn render_tabs(tab: usize, kind: &NodeKind, area: Rect, buf: &mut Buffer) {
    let titles = match kind {
        NodeKind::Drone { .. } => {
            vec!["Sent", "Dropped", "Shortcutted"]
        }
        NodeKind::Client | NodeKind::Server => {
            vec!["Packets Sent", "Messages Sent", "Messages Received"]
        }
    };
    Tabs::new(titles)
        .select(tab)
        .padding("", "")
        .divider(" ")
        .render(area, buf);
}

pub fn render_tab_content(
    tab: usize,
    screen: &Screen,
    network: &Network,
    table_state: &mut TableState,
    area: Rect,
    frame: &mut Frame,
) {
    let node = network.get_node_from_id(screen.focus).unwrap();
    match (tab, node.kind) {
        // all tabs of drone and first tab of client/server
        (1..=2, NodeKind::Client | NodeKind::Server) => {
            let widths = [
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(6),
                Constraint::Length(10),
            ];
            let selected_row_style = Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(TEXT_COLOR);
            let header = Row::new(vec!["typ", "←/→", "src", "sid"]);
            let rows: Vec<Row<'_>> = match tab {
                2 => {
                    let mdeque = node.mreceived.iter();
                    mdeque.map(|p| message_table_row(p, true)).collect()
                }
                1 => {
                    let mdeque = node.msent.iter();

                    mdeque
                        .map(|(_, (m, finished))| message_table_row(m, *finished))
                        .rev()
                        .collect()
                }
                _ => unreachable!(),
            };

            let table = Table::new(rows, widths)
                .column_spacing(1)
                .style(Style::new())
                .row_highlight_style(selected_row_style)
                .header(header.clone());

            frame.render_stateful_widget(table, area, table_state);
        }
        (1..=2, NodeKind::Drone { .. }) | (0, _) => {
            let widths = [
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ];
            let selected_row_style = Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(TEXT_COLOR);
            let header = Row::new(vec!["type", "sid", "src", "dst", "about"]);
            let pdeque = match tab {
                0 => node.sent.iter(),
                1 => node.dropped.iter(),
                2 => node.shortcutted.iter(),
                _ => unreachable!(),
            };
            let rows: Vec<Row<'_>> = pdeque.map(|p| packet_table_row(p)).collect();

            let table = Table::new(rows, widths)
                .column_spacing(1)
                .style(Style::new())
                .row_highlight_style(selected_row_style)
                .header(header.clone());

            //.block(area);
            frame.render_stateful_widget(table, area, table_state);
        }
        _ => unreachable!(),
    }
}
