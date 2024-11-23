use add_connection_handler::handle_keypress_add_connection;
use add_node_handler::handle_keypress_add_node;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use main_handler::handle_keypress_main;
use move_handler::handle_keypress_move;
use crate::{model::{screen::Screen, Model}, utilities::app_message::AppMessage};

mod move_handler;
mod main_handler;
mod add_connection_handler;
mod add_node_handler;


pub fn handle_crossterm_events(model: &mut Model) -> std::io::Result<Option<AppMessage>> {
    match event::read()? {
        // check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => Ok(handle_keypress(model, key)),
        // Event::Mouse(_) => Ok(None),
        // Event::Resize(_, _) => Ok(None),
        _ => Ok(None),
    }
}

/// Handles the key events and updates the state of [`App`].
fn handle_keypress(model: &mut Model, key: KeyEvent) -> Option<AppMessage>{
    match model.screen {
        Screen::Start => None, //handle_keypress_start(model,key),
        Screen::Main => handle_keypress_main(model, key),
        Screen::Move => handle_keypress_move(model, key),
        Screen::AddNode => handle_keypress_add_node(model, key),
        Screen::AddConnection { origin: from } => handle_keypress_add_connection(model, key, from),
    }
}

// fn handle_keypress_start(model:&mut Model, key: KeyEvent) {
//     match (key.modifiers, key.code) {
//         (_, KeyCode::Char('q')) => model.running = false,
//         // (_, KeyCode::Up) => model.node_list_state.scroll_up_by(1),
//         // (_, KeyCode::Down) => model.node_list_state.scroll_down_by(1),
//         (_, KeyCode::Char('+')) => model.screen = Screen::Main,
//         _ => {}
//     }
// }