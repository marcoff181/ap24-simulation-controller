use crossterm::event::{KeyCode, KeyEvent};
use crate::{model::{node_kind::NodeKind, screen::Screen, Model}, utilities::app_message::AppMessage};



pub fn handle_keypress_add_node(model: &mut Model, key: KeyEvent) -> Option<AppMessage>  {
    // when you enter add_node screen, the new node gets selected
    let n = match model.get_mut_selected_node() {
        Some(x) => x,
        None => {
            model.screen = Screen::Main;
            return None;
        }
    };
    // saves keycode enter from the borrow checker
    let id = n.id;

    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => n.shiftu(1),
        (_, KeyCode::Down) => n.shiftd(1),
        (_, KeyCode::Left) => n.shiftl(1),
        (_, KeyCode::Right) => n.shiftr(1),
        (_, KeyCode::Char('c')) => n.kind = NodeKind::Client,
        (_, KeyCode::Char('s')) => n.kind = NodeKind::Server,
        (_, KeyCode::Char('d')) => {
            n.kind = NodeKind::Drone {
                pdr: 0.05,
                crashed: false,
            }
        } //todo: add way to edit or 
        // for now leave it as operation to do after creation
        (_, KeyCode::Enter) => {model.screen=Screen::Main;return Some(AppMessage::AddNode { node: id })},
        (_, KeyCode::Char('q')) => return Some(AppMessage::Quit),
        _ => {}
    };
    None
}