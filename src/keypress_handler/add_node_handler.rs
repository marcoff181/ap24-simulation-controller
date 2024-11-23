use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crate::{model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen, Model}, utilities::app_message::AppMessage};



pub fn handle_keypress_add_node(model: &mut Model, key: KeyEvent) -> Option<AppMessage>  {
    // when you enter add_node screen, the new node gets selected
    let n = match model.node_list_state.selected() {
        None => {
            model.screen = Screen::Main;
            return None;
        }
        Some(x) => &mut model.nodes[x],
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => n.shiftu(1),
        (_, KeyCode::Down) => n.shiftd(1),
        (_, KeyCode::Left) => n.shiftl(1),
        (_, KeyCode::Right) => n.shiftr(1),
        (_, KeyCode::Char('c')) => n.kind = NodeKind::Client,
        (_, KeyCode::Char('s')) => n.kind = NodeKind::Server,
        (_, KeyCode::Char('d')) => {
            n.kind = NodeKind::Drone {
                pdr: 0.5,
                crashed: false,
            }
        } //todo: add way to edit
        (_, KeyCode::Enter) => model.screen = Screen::Main,
        (_, KeyCode::Char('q')) => model.running = false,
        _ => {}
    };
    None
}