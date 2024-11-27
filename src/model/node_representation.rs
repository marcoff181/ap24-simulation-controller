// use std::hash::Hash;

use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use rand::Rng;
use wg_2024::{
    config::{Client, Drone, Server},
    network::NodeId,
    packet::Packet,
};

use super::node_kind::NodeKind;

#[derive(Debug)]
pub struct NodeRepresentation {
    // will have a field with the actual drone
    //todo: do they all need to be pub?
    pub id: NodeId,
    pub x: u32,
    pub y: u32,
    pub kind: NodeKind,
    pub adj: HashSet<NodeId>,
    pub sent: VecDeque<Packet>,
    pub dropped: VecDeque<Packet>,
}

impl PartialEq for NodeRepresentation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NodeRepresentation {}

// there are no nodes with the same id
impl std::hash::Hash for NodeRepresentation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Default for NodeRepresentation {
    fn default() -> Self {
        NodeRepresentation::new(
            // todo: check if there is a node with same id
            rand::thread_rng().gen_range(0..=255),
            0,
            0,
            NodeKind::Drone {
                pdr: 0.0,
                crashed: false,
            },
            HashSet::new(),
        )
    }
}

impl Display for NodeRepresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            NodeKind::Drone { pdr: _, crashed: _ } => write!(f, "{}  #{}", self.kind, self.id),
            NodeKind::Server | NodeKind::Client => write!(f, "{} #{}", self.kind, self.id),
        }
    }
}

impl NodeRepresentation {
    pub fn new(id: NodeId, x: u32, y: u32, kind: NodeKind, adj: HashSet<NodeId>) -> Self {
        let s = format!("{:?} #{}", kind, id);
        NodeRepresentation {
            id,
            x,
            y,
            kind,
            adj,
            sent: VecDeque::new(),
            dropped: VecDeque::new(),
        }
    }

    pub fn new_from_cfgdrone(d: &Drone) -> Self {
        NodeRepresentation::new(
            d.id,
            rand::thread_rng().gen_range(0..=100),
            rand::thread_rng().gen_range(0..=100),
            NodeKind::Drone {
                pdr: d.pdr,
                crashed: false,
            },
            d.connected_node_ids.iter().cloned().collect(),
        )
    }

    pub fn new_from_cfgclient(d: &Client) -> Self {
        NodeRepresentation::new(
            d.id,
            rand::thread_rng().gen_range(0..=100),
            rand::thread_rng().gen_range(0..=100),
            NodeKind::Client,
            d.connected_drone_ids.iter().cloned().collect(),
        )
    }

    pub fn new_from_cfgserver(d: &Server) -> Self {
        NodeRepresentation::new(
            d.id,
            rand::thread_rng().gen_range(0..=100),
            rand::thread_rng().gen_range(0..=100),
            NodeKind::Server,
            d.connected_drone_ids.iter().cloned().collect(),
        )
    }

    pub fn shiftr(&mut self, offset: u32) {
        self.x = self.x.saturating_add(offset);
    }

    pub fn shiftl(&mut self, offset: u32) {
        self.x = self.x.saturating_sub(offset);
    }

    pub fn shiftu(&mut self, offset: u32) {
        self.y = self.y.saturating_add(offset);
    }

    pub fn shiftd(&mut self, offset: u32) {
        self.y = self.y.saturating_sub(offset);
    }
}
