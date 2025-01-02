use crossterm::event::KeyCode;
use wg_2024::network::NodeId;

use crate::utilities::app_message::AppMessage;

#[derive(Debug, Default)]
pub enum Screen {
    Start,
    #[default]
    Main,
    Move,
    AddNode,
    AddConnection {
        origin: NodeId,
    },
    //Detail {
    //    of: NodeId,
    //},
}

pub struct Keydef<'a> {
    icon: &'a str,
    desc: &'a str,
    code: KeyCode,
    action: AppMessage,
}
pub const MOVE_KEYS: [Keydef<'_>; 1] = [Keydef {
    icon: "d",
    desc: "b",
    code: KeyCode::Esc,
    action: AppMessage::Quit,
}];
