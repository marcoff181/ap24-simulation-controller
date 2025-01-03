use wg_2024::network::NodeId;

use crate::network::{node_kind::NodeKind, node_representation::NodeRepresentation};

#[derive(Debug)]
pub enum Window {
    AddConnection { origin: NodeId },
    AddNode { toadd: NodeRepresentation },
    ChangePdr { pdr: f64 },
    Detail,
    Main,
    Move,
}

pub struct Screen {
    pub focus: NodeId,
    pub kind: NodeKind,
    pub window: Window,
}
