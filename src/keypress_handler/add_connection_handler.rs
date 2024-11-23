use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use wg_2024::network::NodeId;
use crate::{model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen, Model}, utilities::app_message::AppMessage};

pub fn handle_keypress_add_connection(model: &mut Model, key: KeyEvent, from: u32) -> Option<AppMessage> {
    let x = match model.node_list_state.selected() {
        None => {
            model.screen = Screen::Main;
            return None;
        }
        Some(x) => x,
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => model.running = false,
        (_, KeyCode::Up) => model.node_list_state.scroll_up_by(1),
        (_, KeyCode::Down) => model.node_list_state.scroll_down_by(1),
        (_, KeyCode::Enter) => {
            return Some(AppMessage::AddConnection { from: from as NodeId, to: x as NodeId });
        }
        _ => {}
    };
    None
}