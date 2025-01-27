use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use messages::node::Node;
use node_kind::NodeKind;
use node_representation::NodeRepresentation;
use wg_2024::{config::Config, network::NodeId, packet::PacketType};

pub mod node_kind;
pub mod node_representation;

#[derive(Debug, Default)]
pub struct Network {
    //todo: some of these don't need to be public
    pub nodes: Vec<NodeRepresentation>,
    pub edges: HashMap<(NodeId, NodeId), Option<(PacketType, Instant)>>,
}
impl Network {
    pub fn new(cfg: &Config) -> Self {
        let nodes: Vec<NodeRepresentation> = Vec::new();
        let edges = HashMap::new();

        let mut model = Self { nodes, edges };

        for d in cfg.drone.iter() {
            model.nodes.push(NodeRepresentation::new_from_cfgdrone(d));
            for to in d.connected_node_ids.iter() {
                model.add_edge(d.id, *to);
            }
        }
        for s in cfg.server.iter() {
            model.nodes.push(NodeRepresentation::new_from_cfgserver(s));
            for to in s.connected_drone_ids.iter() {
                model.add_edge(s.id, *to);
            }
        }
        for c in cfg.client.iter() {
            model.nodes.push(NodeRepresentation::new_from_cfgclient(c));
            for to in c.connected_drone_ids.iter() {
                model.add_edge(c.id, *to);
            }
        }
        model
    }

    /// adds a new edge, updating bot `self.edges` and `node.adj`
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        match from.cmp(&to) {
            std::cmp::Ordering::Less => {
                self.edges.insert((from, to), None);
            }
            std::cmp::Ordering::Greater => {
                self.edges.insert((to, from), None);
            }
            // node can't have edge that points to itself
            std::cmp::Ordering::Equal => {}
        };

        // todo: decide if there is need to keep this logic related to adj
        if let Some(nodefrom_id) = self.get_mut_node_from_id(from) {
            nodefrom_id.adj.insert(to as NodeId);
        }

        if let Some(nodeto_id) = self.get_mut_node_from_id(to) {
            nodeto_id.adj.insert(from as NodeId);
        }
    }

    /// updates existing edge, with the last packet that has traveled on it
    pub fn update_edge_activity(&mut self, from: NodeId, to: NodeId, packet_passed: PacketType) {
        if self.edges.contains_key(&(from, to)) {
            self.edges
                .insert((from, to), Some((packet_passed, Instant::now())));
        }
    }

    /// if present returns immutable reference to the drone at the given `idx` of the nodes vector
    pub fn get_node_from_pos(&self, idx: usize) -> Option<&NodeRepresentation> {
        if idx < self.nodes.len() {
            Some(&self.nodes[idx])
        } else {
            None
        }
    }

    /// if present returns immutable reference to the drone with the corresponding `id`
    pub fn get_node_from_id(&self, id: NodeId) -> Option<&NodeRepresentation> {
        self.nodes.iter().find(|&node| node.id == id)
    }

    /// if present returns mutable reference to the drone with the corresponding `id`
    pub fn get_mut_node_from_id(&mut self, id: NodeId) -> Option<&mut NodeRepresentation> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    /// sets drone with matching id `crashed` parameter to true,
    /// then removes all edges that contain the drone
    pub fn crash_drone(&mut self, id: NodeId) {
        if let Some(drone) = self.get_mut_node_from_id(id) {
            drone.kind = match drone.kind {
                NodeKind::Drone { pdr, crashed: _ } => NodeKind::Drone { pdr, crashed: true },
                other => other,
            }
        }

        self.edges.retain(|(from, to), _| *from != id && *to != id);
    }
}
