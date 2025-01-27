use wg_2024::network::NodeId;

use crate::network::node_kind::NodeKind;

#[derive(Debug)]
/// contains all the different gui states, meaning, different windows or popups
pub enum Window {
    AddConnection { origin: NodeId },
    ChangePdr { pdr: f32 },
    Detail { tab: usize },
    Error { message: &'static str },
    Main,
    Move,
}

/// contains all information about the state of the gui, and the currently selected node
pub struct Screen {
    pub focus: NodeId,
    pub kind: NodeKind,
    pub window: Window,
}
