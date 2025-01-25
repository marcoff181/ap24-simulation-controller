mod draw_options;
mod footer;
mod keys;
mod list;
mod node_detail;
mod packet_formatter;
mod simulation;
mod stats;
mod tabs;

use footer::render_footer;
use layout::Flex;
use list::render_list;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Clear, Gauge, ListState, Padding, Paragraph, TableState, Wrap,
};
use simulation::render_simulation;
use stats::render_stats;
//use wg_2024::config::{Client, Drone, Server};

use crate::network::node_kind::NodeKind;
use crate::network::Network;
use crate::screen::Window;
use crate::utilities::theme::{BG_COLOR, CRASH_COLOR, HIGHLIGHT_COLOR, TEXT_COLOR};
use crate::Screen;

pub fn render(
    network: &Network,
    screen: &Screen,
    node_list_state: &mut ListState,
    table_state: &mut TableState,
    frame: &mut Frame,
) {
    let [main, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    render_footer(network, screen, footer, frame.buffer_mut());

    match screen.window {
        Window::Error { message } => {
            render_error(message, main, frame);
        }
        Window::Detail { tab } => {
            render_detail(network, tab, screen, table_state, main, frame);
        }
        Window::Main | Window::Move | Window::AddConnection { origin: _ } => {
            render_standard(network, screen, node_list_state, main, frame);
        }
        Window::ChangePdr { pdr } => render_changepdr(pdr, main, frame),
    }
}

fn render_changepdr(pdr: f32, area: Rect, frame: &mut Frame) {
    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Max(5), Constraint::Fill(1)]);
    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Fill(10),
        Constraint::Fill(1),
    ]);
    let [_, area, _] = vertical.areas(area);
    let [_, area, _] = horizontal.areas(area);

    let mut block = Block::bordered();
    let inner = block.inner(area);
    //format!("PDR: {:.2}", pdr)
    let mut gauge = Gauge::default();
    if pdr == 1.0 {
        gauge = gauge.label(Span::from("🦆🦆🦆🦆🦆🦆🦆🦆🦆".to_string()));
        gauge = gauge.gauge_style(CRASH_COLOR).ratio(pdr as f64);
        block = block.border_style(Style::default().fg(CRASH_COLOR));
    } else {
        gauge = gauge.label(Span::from(format!("PDR: {:.2}", pdr)));
        gauge = gauge.gauge_style(HIGHLIGHT_COLOR).ratio(pdr as f64);
        block = block.border_style(Style::default().fg(TEXT_COLOR));
    }

    frame.render_widget(block, area);
    gauge.render(inner, frame.buffer_mut());
}
fn render_error(message: &str, area: Rect, frame: &mut Frame) {
    let vertical = Layout::vertical([Constraint::Fill(1), Constraint::Max(5), Constraint::Fill(1)]);
    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Max(50),
        Constraint::Fill(1),
    ]);
    let [_, area, _] = vertical.areas(area);
    let [_, area, _] = horizontal.areas(area);

    let block = Block::bordered()
        .title("Error")
        .border_style(Style::default().fg(CRASH_COLOR))
        .title_style(Style::default().fg(CRASH_COLOR));

    let inner = block.inner(area);
    Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(TEXT_COLOR))
        .render(inner, frame.buffer_mut());
    frame.render_widget(block, area);
}

fn render_detail(
    network: &Network,
    tab: usize,
    screen: &Screen,
    table_state: &mut TableState,
    area: Rect,
    frame: &mut Frame,
) {
    let [tabs, area] = Layout::vertical([Constraint::Max(1), Constraint::Fill(1)]).areas(area);
    let [left, right] = Layout::horizontal([Constraint::Max(16), Constraint::Fill(1)]).areas(area);

    let [top, bottom] = Layout::vertical([Constraint::Fill(3), Constraint::Fill(1)]).areas(right);

    let left_border_set = symbols::border::Set {
        top_right: symbols::line::NORMAL.horizontal_down,
        bottom_right: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };
    let top_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        top_right: symbols::line::NORMAL.top_right,
        bottom_right: symbols::line::NORMAL.vertical_left,
        bottom_left: symbols::line::NORMAL.vertical_right,
        ..symbols::border::PLAIN
    };
    let bottom_border_set = symbols::border::Set {
        top_right: symbols::line::NORMAL.vertical_left,
        top_left: symbols::line::NORMAL.vertical_right,
        bottom_right: symbols::line::NORMAL.bottom_right,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    let leftborder = Block::new()
        .border_set(left_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);
    let topborder = Block::new()
        .border_set(top_border_set)
        .borders(Borders::ALL)
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);
    let bottomborder = Block::new()
        .border_set(bottom_border_set)
        .borders(Borders::RIGHT | Borders::LEFT | Borders::BOTTOM)
        .bg(BG_COLOR)
        .fg(TEXT_COLOR);

    let left_inner = leftborder.inner(left);
    let top_inner = topborder.inner(top);
    let bottom_inner = bottomborder.inner(bottom);

    leftborder.render(left, frame.buffer_mut());
    topborder.render(top, frame.buffer_mut());
    bottomborder.render(bottom, frame.buffer_mut());

    tabs::render_tabs(tab, &screen.kind, tabs, frame.buffer_mut());
    tabs::render_tab_content(tab, screen, network, table_state, left_inner, frame);

    let node = network.get_node_from_id(screen.focus).unwrap();

    node_detail::node_detail(node, top_inner, frame);

    if tab == 0 || matches!(screen.kind, NodeKind::Drone { .. }) {
        let packet = match (tab, screen.kind) {
            (0, _) => node.sent.get(table_state.selected().unwrap_or(usize::MAX)),
            (1, NodeKind::Drone { .. }) => node
                .dropped
                .get(table_state.selected().unwrap_or(usize::MAX)),
            (2, NodeKind::Drone { .. }) => node
                .shortcutted
                .get(table_state.selected().unwrap_or(usize::MAX)),
            _ => None,
        };

        let t = match packet {
            Some(p) => packet_formatter::packet_detail(p),
            None => Paragraph::default(),
        };
        t.render(bottom_inner, frame.buffer_mut());
    } else {
        let t = match (tab, screen.kind) {
            (1, NodeKind::Client | NodeKind::Server) => {
                if let Some((_, (m, _))) = node.msent.get_index(
                    node.msent
                        .len()
                        .saturating_sub(table_state.selected().unwrap_or(usize::MAX) + 1),
                ) {
                    packet_formatter::message_detail(m)
                } else {
                    Paragraph::default()
                }
            }
            (2, NodeKind::Client | NodeKind::Server) => {
                if let Some(message) = node
                    .mreceived
                    .get(table_state.selected().unwrap_or(usize::MAX))
                {
                    packet_formatter::message_detail(message)
                } else {
                    Paragraph::default()
                }
            }
            _ => Paragraph::default(),
        };
        t.render(bottom_inner, frame.buffer_mut());
    };
}

fn render_standard(
    network: &Network,
    screen: &Screen,
    node_list_state: &mut ListState,
    area: Rect,
    frame: &mut Frame,
) {
    let [top, bottom] =
        Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]).areas(area);

    let [left, right] = Layout::horizontal([Constraint::Max(14), Constraint::Fill(1)]).areas(top);

    let top_right_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        ..symbols::border::PLAIN
    };

    let block = Block::new()
        .border_set(top_right_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title("Simulation")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR)
        .padding(Padding::proportional(1));

    let inner_right = block.inner(right);

    block.render(right, frame.buffer_mut());
    render_simulation(
        crate::view::draw_options::DrawGraphOptions::from_network(network, screen),
        inner_right,
        frame.buffer_mut(),
    );
    render_stats(network, screen, bottom, frame);
    render_list(network, screen, node_list_state, left, frame.buffer_mut());
}
