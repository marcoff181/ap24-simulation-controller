mod footer;
mod keys;
mod list;
mod packet_formatter;
mod simulation;
mod stats;

use footer::render_footer;
use list::render_list;
use ratatui::buffer::Buffer;
use ratatui::prelude::*;
use simulation::render_simulation;
use stats::render_stats;

use crate::model::screen::Screen;
use crate::model::Model;

pub fn render(model: &mut Model, frame: &mut Frame) {
    let [main, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    render_footer(model, footer, frame.buffer_mut());

    match model.screen {
        Screen::Start => {
            //render_start(main, buf);
        }
        Screen::Main | Screen::Move | Screen::AddConnection { origin: _ } | Screen::AddNode => {
            render_default(model, main, frame);
        }
    }
}

fn render_default(model: &mut Model, area: Rect, frame: &mut Frame) {
    let [left, right] = Layout::horizontal([Constraint::Max(14), Constraint::Fill(1)]).areas(area);

    let [top_right, bottom_right] =
        Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]).areas(right);

    render_list(model, left, frame.buffer_mut());
    render_simulation(model, top_right, frame.buffer_mut());
    render_stats(model, bottom_right, frame);
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
