use crossterm::event::{KeyCode, KeyEvent};
use wg_2024::network::NodeId;
use crate::{model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen, Model}, utilities::app_message::AppMessage};



pub fn handle_keypress_main(model: &mut Model, key: KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => model.node_list_state.scroll_up_by(1),
        (_, KeyCode::Down) => model.node_list_state.scroll_down_by(1),
        (_, KeyCode::Char('q')) => return Some(AppMessage::Quit),
        (_, KeyCode::Char('m')) => model.screen = Screen::Move,
        (_, KeyCode::Char('c')) => {
            model.screen = Screen::AddConnection {
                origin: model.node_list_state.selected().unwrap() as NodeId,
            }
        }
        (_, KeyCode::Char('+')) => {
            model.nodes.push(NodeRepresentation::default());
            model.node_list_state.select_last();
            model.screen = Screen::AddNode
        }
        other => match model.get_selected_kind() {
            Some(NodeKind::Drone { pdr: _, crashed: _ }) => match other {
                (_, KeyCode::Char('p')) => todo!(),
                (_, KeyCode::Char('k')) => todo!(),
                _ => {}
            },
            _ => {}
        },
    };

    None
}