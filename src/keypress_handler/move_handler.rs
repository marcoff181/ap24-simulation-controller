use crate::{
    model::{screen::Screen, Model},
    utilities::app_message::AppMessage,
};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_keypress_move(model: &mut Model, key: KeyEvent) -> Option<AppMessage> {
    let n = match model.node_list_state.selected() {
        None => {
            model.screen = Screen::Main;
            return None;
        }
        Some(x) => &mut model.nodes[x],
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => return Some(AppMessage::Quit),
        (_, KeyCode::Up) => n.shiftu(1),
        (_, KeyCode::Down) => n.shiftd(1),
        (_, KeyCode::Left) => n.shiftl(1),
        (_, KeyCode::Right) => n.shiftr(1),
        (_, KeyCode::Enter) => model.screen = Screen::Main,
        _ => {}
    };
    None
}
