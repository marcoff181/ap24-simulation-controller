use wg_2024::network::NodeId;

use crate::network::node_kind::NodeKind;

#[derive(Debug)]
pub enum Window {
    AddConnection { origin: NodeId },
    ChangePdr { pdr: f32 },
    Detail { tab: usize },
    Error { message: &'static str },
    Main,
    Move,
}

pub struct Screen {
    pub focus: NodeId,
    pub kind: NodeKind,
    pub window: Window,
}
