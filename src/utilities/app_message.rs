use wg_2024::packet::NodeType;

use crate::network::node_kind::NodeKind;

pub enum AppMessage {
    // used in move node and add node
    MoveNode { x: i8, y: i8 },

    // used in main
    WindowAddConnection,
    WindowAddNode,
    WindowChangePDR,
    WindowMove,
    WindowDetail,
    Crash,

    // used in main, add connection
    ScrollUp,
    ScrollDown,

    // used in add connection, add node, Detail, Move, Changepdr
    Done,

    // used in add node
    SetNodeKind(NodeKind),

    // used in all
    Quit,
}
