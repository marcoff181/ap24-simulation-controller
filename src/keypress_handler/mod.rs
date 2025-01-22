use std::time::Duration;

use crate::network::node_kind::NodeKind;
use crate::screen::{Screen, Window};
use crate::utilities::app_message::AppMessage;
use crossterm::event::KeyCode;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

pub fn handle_crossterm_events(screen: &Screen) -> std::io::Result<Option<AppMessage>> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            // check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => Ok(handle_keypress(screen, &key)),
            // Event::Mouse(_) => Ok(None),
            // Event::Resize(_, _) => Ok(None),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// Handles the key events and returns an AppMessage defining the action that is requested
fn handle_keypress(screen: &Screen, key: &KeyEvent) -> Option<AppMessage> {
    match screen.window {
        Window::AddConnection { origin: _ } => handle_keypress_add_connection(key),
        Window::AddNode { toadd: _ } => handle_keypress_add_node(key),
        Window::ChangePdr { pdr: _ } => handle_keypress_changepdr(key),
        Window::Detail { tab: _ } => handle_keypress_detail(key),
        Window::Main => handle_keypress_main(screen.kind, key),
        Window::Move => handle_keypress_move(key),
    }
}

pub fn handle_keypress_changepdr(key: &KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        //(_, KeyCode::Char('q')) => Some(AppMessage::Quit),
        //(_, KeyCode::Up) => Some(AppMessage::MoveNode { x: 1, y: 0 }),
        //(_, KeyCode::Down) => Some(AppMessage::MoveNode { x: -1, y: 0 }),
        //(_, KeyCode::Left) => Some(AppMessage::MoveNode { x: 0, y: 1 }),
        //(_, KeyCode::Right) => Some(AppMessage::MoveNode { x: 0, y: -1 }),
        //(_, KeyCode::Enter) => Some(AppMessage::Done),
        _ => None,
    }
}

pub fn handle_keypress_detail(key: &KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => Some(AppMessage::ScrollUp),
        (_, KeyCode::Down) => Some(AppMessage::ScrollDown),
        (_, KeyCode::Tab) => Some(AppMessage::ChangeTab),
        (_, KeyCode::Enter) => Some(AppMessage::Done),
        (_, KeyCode::Char('q')) => Some(AppMessage::Quit),
        _ => None,
    }
}

pub fn handle_keypress_main(sel_type: NodeKind, key: &KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => Some(AppMessage::ScrollUp),
        (_, KeyCode::Down) => Some(AppMessage::ScrollDown),
        (_, KeyCode::Char('q')) => Some(AppMessage::Quit),
        (_, KeyCode::Char('m')) => Some(AppMessage::WindowMove),
        (_, KeyCode::Char('c')) => Some(AppMessage::WindowAddConnection),
        (_, KeyCode::Char('+')) => Some(AppMessage::WindowAddNode),
        (_, KeyCode::Char('d')) => Some(AppMessage::WindowDetail),
        (_, KeyCode::Char('p')) if matches!(sel_type, NodeKind::Drone { .. }) => {
            Some(AppMessage::WindowChangePDR)
        }
        (_, KeyCode::Char('k')) if matches!(sel_type, NodeKind::Drone { .. }) => {
            Some(AppMessage::Crash)
        }
        _ => None,
    }
}

pub fn handle_keypress_move(key: &KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => Some(AppMessage::MoveNode { x: 0, y: 1 }),
        (_, KeyCode::Down) => Some(AppMessage::MoveNode { x: 0, y: -1 }),
        (_, KeyCode::Left) => Some(AppMessage::MoveNode { x: -1, y: 0 }),
        (_, KeyCode::Right) => Some(AppMessage::MoveNode { x: 1, y: 0 }),
        (_, KeyCode::Char('q')) => Some(AppMessage::Quit),
        (_, KeyCode::Enter) => Some(AppMessage::Done),
        _ => None,
    }
}

pub fn handle_keypress_add_node(key: &KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => Some(AppMessage::MoveNode { x: 0, y: 1 }),
        (_, KeyCode::Down) => Some(AppMessage::MoveNode { x: 0, y: -1 }),
        (_, KeyCode::Left) => Some(AppMessage::MoveNode { x: -1, y: 0 }),
        (_, KeyCode::Right) => Some(AppMessage::MoveNode { x: 1, y: 0 }),
        (_, KeyCode::Char('c')) => Some(AppMessage::SetNodeKind(NodeKind::Client)),
        (_, KeyCode::Char('s')) => Some(AppMessage::SetNodeKind(NodeKind::Server)),
        (_, KeyCode::Char('d')) => Some(AppMessage::SetNodeKind(NodeKind::Drone {
            pdr: 0.05,
            crashed: false,
        })),
        (_, KeyCode::Enter) => Some(AppMessage::Done),
        (_, KeyCode::Char('q')) => Some(AppMessage::Quit),
        _ => None,
    }
}

pub fn handle_keypress_add_connection(key: &KeyEvent) -> Option<AppMessage> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => Some(AppMessage::Quit),
        (_, KeyCode::Up) => Some(AppMessage::ScrollUp),
        (_, KeyCode::Down) => Some(AppMessage::ScrollDown),
        (_, KeyCode::Enter) => Some(AppMessage::Done),
        _ => None,
    }
}
