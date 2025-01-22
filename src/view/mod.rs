mod footer;
mod keys;
mod list;
mod packet_formatter;
mod simulation;
mod stats;
mod tabs;

use footer::render_footer;
use layout::Flex;
use list::render_list;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, ListState, Padding, Paragraph, TableState, Wrap};
use simulation::render_simulation;
use stats::render_stats;
//use wg_2024::config::{Client, Drone, Server};

use crate::network::node_kind::NodeKind;
use crate::network::Network;
use crate::screen::Window;
use crate::utilities::theme::{BG_COLOR, CRASH_COLOR, TEXT_COLOR};
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
        Window::Main
        | Window::ChangePdr { pdr: _ }
        | Window::Move
        | Window::AddConnection { origin: _ } => {
            render_standard(network, screen, node_list_state, main, frame);
        }
    }
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

    let [top, bottom] =
        Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)]).areas(right);

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
    let packet = match (tab, screen.kind) {
        (0, _) => node.sent.get(table_state.selected().unwrap()),
        (1, NodeKind::Drone { .. }) => node.dropped.get(table_state.selected().unwrap()),
        (2, NodeKind::Drone { .. }) => node.shortcutted.get(table_state.selected().unwrap()),
        _ => None,
    };

    let t = packet_formatter::packet_detail(packet.unwrap());
    t.render(bottom_inner, frame.buffer_mut());
    //match kind {
    //    NodeKind::Client | NodeKind::Server => {
    //        let [left, right] =
    //            Layout::horizontal([Constraint::Max(14), Constraint::Fill(1)]).areas(top);
    //    }
    //    NodeKind::Drone { pdr: _, crashed: _ } => {
    //        let [left, right] =
    //            Layout::horizontal([Constraint::Max(14), Constraint::Fill(1)]).areas(top);
    //    }
    //}
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

    render_simulation(network, screen, right, frame.buffer_mut());
    render_stats(network, screen, bottom, frame);
    render_list(network, screen, node_list_state, left, frame.buffer_mut());
}

// fn render_start(model:&Model, area: Rect, buf: &mut Buffer) {
//     let block = Block::new()
//         .borders(Borders::ALL)
//         // .title("Simulation Controller")
//         .bg(BG_COLOR)
//         .fg(TEXT_COLOR);

//     let big_text = BigText::builder()
//         .centered()
//         .pixel_size(PixelSize::Sextant)
//         .style(Style::new().blue().bg(BG_COLOR))
//         .lines(vec![
//             "Simulation".red().into(),
//             "Controller".white().into(),
//             // "by marcoff181".into(),
//         ])
//         .build();

//     // Get the inner area of the block to render the BigText
//     let inner_area = block.inner(area);

//     let [top, bottom] =
//         Layout::vertical([Constraint::Max(6), Constraint::Fill(1)]).areas(inner_area);

//     // horiz crop
//     // let [h,_] = Layout::horizontal([Constraint::Length(width), Constraint::Min(0)]).areas(bottom);
//     //vert crop
//     // let [hv,_] = Layout::horizontal([Constraint::Length(height), Constraint::Min(0)]).areas(h);

//     // let mut text:String = String::new();

//     big_text.render(top, buf);
//     block.render(area, buf);

//     //render_image(bottom, buf, "./media/pixil-frame-0.png");
// }

// fn render_footer(model:&Model, area: Rect, buf: &mut Buffer) {
//     let start_keys = [
//         // ("↑", "Up"),
//         // ("↓", "Down"),
//         ("+", "Open initialization file"),
//         ("q", "Quit"),
//     ];

//     let main_keys = [
//         ("↑/↓", "Scroll list"),
//         ("m", "Move node"),
//         ("c", "Add connection"),
//         ("+", "Add node"),
//         ("q", "Quit"),
//     ];

//     let main_keys_over_drone = [
//         ("↑/↓", "Scroll list"),
//         ("m", "Move node"),
//         ("c", "Add connection"),
//         ("+", "Add node"),
//         ("p", "Edit PDR"),
//         ("k", "Crash"),
//         ("q", "Quit"),
//     ];

//     let main_keys_add_connection = [
//         ("↑/↓", "Scroll list"),
//         ("Enter", "Connect to selected node"),
//         ("q", "Quit"),
//     ];

//     let main_keys_add_node = [
//         ("↑/↓/→/←", "Move"),
//         ("s/c/d", "Set drone type"),
//         ("Enter", "Add node"),
//         ("q", "Quit"),
//     ];

//     let move_keys = [("↑/↓/→/←", "Move"), ("Enter", "Ok"), ("q", "Quit")];

//     let keys: &[(&str, &str)] = match screen {
//         Screen::Main => match get_selected_kind() {
//             Some(NodeKind::Drone) => &main_keys_over_drone,
//             _ => &main_keys,
//         },
//         Screen::Start => &start_keys,
//         Screen::Move => &move_keys,
//         Screen::AddConnection { origin: _ } => &main_keys_add_connection,
//         Screen::AddNode => &main_keys_add_node,
//     };

//     let spans: Vec<Span> = keys
//         .iter()
//         .flat_map(|(key, desc)| {
//             let key = Span::styled(
//                 format!(" {key} "),
//                 Style::new().fg(INVERTED_TEXT_COLOR).bg(HIGHLIGHT_COLOR),
//             );
//             let desc = Span::styled(
//                 format!(" {desc} "),
//                 Style::new().fg(TEXT_COLOR).bg(BOTTOMPANEL_BG),
//             );
//             [key, desc]
//         })
//         .collect();

//     Line::from(spans)
//         .centered()
//         .style((Color::Yellow, BOTTOMPANEL_BG))
//         .render(area, buf);
// }
