use crossterm::event::{KeyCode, KeyEvent};
use wg_2024::network::NodeId;
use crate::{model::{screen::Screen, Model}, utilities::app_message::AppMessage};

pub fn handle_keypress_add_connection(model: &mut Model, key: KeyEvent, from: NodeId) -> Option<AppMessage> {
    let x = match model.node_list_state.selected() {
        None => {
            model.screen = Screen::Main;
            return None;
        }
        Some(x) => x,
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => return Some(AppMessage::Quit),
        (_, KeyCode::Up) => model.node_list_state.scroll_up_by(1),
        (_, KeyCode::Down) => model.node_list_state.scroll_down_by(1),
        (_, KeyCode::Enter) => {
            return Some(AppMessage::AddConnection { from: from as NodeId, to: x as NodeId });
        }
        _ => {}
    };
    None
}