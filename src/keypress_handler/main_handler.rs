use crate::{
    model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen, Model},
    utilities::app_message::AppMessage,
};
use crossterm::event::{KeyCode, KeyEvent};
use wg_2024::network::NodeId;

pub fn handle_keypress_main(model: &mut Model, key: KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => model.node_list_state.scroll_up_by(1),
        (_, KeyCode::Down) => model.node_list_state.scroll_down_by(1),
        (_, KeyCode::Char('q')) => return Some(AppMessage::Quit),
        (_, KeyCode::Char('m')) => model.screen = Screen::Move,
        (_, KeyCode::Char('c')) => {
            model.screen = Screen::AddConnection {
                origin: model.selected_node_id().unwrap(),
            }
        }
        (_, KeyCode::Char('+')) => {
            model.spawn_default_node();
            model.screen = Screen::AddNode
        }
        other => {
            if let Some(NodeKind::Drone { pdr: _, crashed: _ }) = model.get_selected_kind() {
                match other {
                    (_, KeyCode::Char('p')) => todo!(),
                    (_, KeyCode::Char('k')) => {
                        if let Some(node) = model.get_selected_node() {
                            return Some(AppMessage::Crash { drone: node.id });
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    None
}
