use core::panic;
use std::{
    borrow::BorrowMut,
    collections::{HashMap, HashSet},
    time::Instant,
};

use node_kind::NodeKind;
use node_representation::NodeRepresentation;
use wg_2024::{config::Config, network::NodeId, packet::PacketType};

pub mod node_kind;
pub mod node_representation;

#[derive(Debug, Default)]
pub struct Network {
    pub nodes: Vec<NodeRepresentation>,
    pub edges: HashMap<(NodeId, NodeId), Option<(PacketType, Instant)>>,
}

impl Network {
    /// creates a network from the given config, populating nodes and edges accordingly, and then
    /// checking if the resulting network is valid
    pub fn new(cfg: &Config) -> Result<Self, &'static str> {
        let nodes: Vec<NodeRepresentation> = Vec::new();
        let edges = HashMap::new();

        let mut model = Self { nodes, edges };

        for d in &cfg.drone {
            model.nodes.push(NodeRepresentation::new_from_cfgdrone(d));
        }
        for s in &cfg.server {
            model.nodes.push(NodeRepresentation::new_from_cfgserver(s));
        }
        for c in &cfg.client {
            model.nodes.push(NodeRepresentation::new_from_cfgclient(c));
        }

        for d in &cfg.drone {
            for to in &d.connected_node_ids {
                let _ = model.add_edge_unchecked(d.id, *to);
            }
        }
        for s in &cfg.server {
            for to in &s.connected_drone_ids {
                let _ = model.add_edge_unchecked(s.id, *to);
            }
        }
        for c in &cfg.client {
            for to in &c.connected_drone_ids {
                let _ = model.add_edge_unchecked(c.id, *to);
            }
        }
        match model.is_valid() {
            Ok(()) => Ok(model),
            Err(s) => Err(s),
        }
    }

    /// check that the current state of the network respects WG rules, if not, returns a
    /// descriptive error
    fn is_valid(&mut self) -> Result<(), &'static str> {
        use NodeKind::{Client, Drone, Server};

        let mut node_ids = HashSet::new();
        let mut drones = HashSet::new();
        let mut graph = HashMap::new();

        let mut crashed_drones = HashSet::new();
        for node in &self.nodes {
            if let Drone { crashed: true, .. } = node.kind {
                crashed_drones.insert(node.id);
            }
        }

        // Build the graph and collect valid drones
        for node in &self.nodes {
            // Skip crashed drones
            if let Drone { crashed: true, .. } = node.kind {
                continue; // Ignore crashed drones
            }

            // Rule 1: A node cannot have itself in its adjacency list
            if node.adj.contains(&node.id) {
                return Err("A node cannot have itself in its adjacency list");
            }

            // Rule 5: No duplicate node IDs
            if !node_ids.insert(node.id) {
                return Err("Duplicate node_id found in Network Initialization File");
            }

            // Collect drones for later verification, ignoring crashed drones
            if matches!(node.kind, Drone { .. }) {
                drones.insert(node.id);
            }

            // Build adjacency graph, excluding crashed drones
            graph.insert(
                node.id,
                node.adj
                    .iter()
                    .filter(|x| !crashed_drones.contains(x))
                    .copied()
                    .collect::<HashSet<u8>>(),
            );
        }

        for node in &self.nodes {
            let mut drone_count = 0;

            // Skip crashed drones during the validation phase
            if let Drone { crashed: true, .. } = node.kind {
                continue; // Ignore crashed drones
            }

            for &neighbor in graph.get(&node.id).unwrap() {
                match node.kind {
                    Client => {
                        // Rule 2 & 3: Clients can only connect to 1-2 drones, not clients or servers
                        if !drones.contains(&neighbor) {
                            return Err("Client cannot connect to other clients or servers");
                        }
                        drone_count += 1;
                    }
                    Server => {
                        // Rule 4: Servers must connect to at least two drones, not clients or servers
                        if !drones.contains(&neighbor) {
                            return Err("Server cannot connect to other clients or servers");
                        }
                        drone_count += 1;
                    }
                    Drone { .. } => {}
                }
            }

            if let Client = node.kind {
                if !(1..=2).contains(&drone_count) {
                    return Err("Client must connect to at least one and at most two drones");
                }
            }

            if let Server = node.kind {
                if drone_count < 2 {
                    return Err("Server must connect to at least two drones");
                }
            }
        }

        // Rule 6: Graph must be connected and bidirectional
        if !Self::is_connected(&graph) {
            return Err("network must represent a connected and bidirectional graph");
        }

        // Rule 7: Removing clients and servers should still leave a connected graph

        // Remove non-drone nodes, ignoring crashed drones
        graph.retain(|&id, _| drones.contains(&id));

        // Remove non-drone connections, ignoring crashed drones
        for adj in graph.values_mut() {
            adj.retain(|id| drones.contains(id));
        }

        if !Self::is_connected(&graph) {
            return Err("clients and servers should be at the edges of the network");
        }

        Ok(())
    }

    /// private helper function to check if a given graph is connected
    fn is_connected(graph: &HashMap<NodeId, HashSet<u8>>) -> bool {
        if graph.is_empty() {
            return false;
        }

        let start = *graph.keys().next().unwrap();
        let mut visited = HashSet::new();
        let mut stack = vec![start];

        while let Some(node) = stack.pop() {
            if visited.insert(node) {
                if let Some(neighbors) = graph.get(&node) {
                    for n in neighbors {
                        if !visited.contains(n) {
                            stack.push(*n);
                        }
                    }
                }
            }
        }

        visited.len() == graph.len()
    }

    //tries to add edge, check if it results in a valid network, if it doesn't it puts network to
    //previous state and returns a string describing what was invalid in the network
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), &'static str> {
        match self.add_edge_unchecked(from, to) {
            Ok(true) => Err("trying to add existing edge"),
            Ok(false) => self
                .is_valid()
                .inspect_err(|_| self.remove_edge_unchecked(from, to)),
            Err(s) => Err(s),
        }
    }

    /// removes edge, updating both `self.edges` and `node.adj`
    fn remove_edge_unchecked(&mut self, from: NodeId, to: NodeId) {
        self.edges.remove(&(from, to));
        self.edges.remove(&(to, from));

        if let Some(nodefrom_id) = self.get_mut_node_from_id(from) {
            nodefrom_id.adj.remove(&to);
        }

        if let Some(nodeto_id) = self.get_mut_node_from_id(to) {
            nodeto_id.adj.remove(&from);
        }
    }

    /// adds a new edge, updating both `self.edges` and `node.adj`, returns Ok(true) if it has been
    /// asked to add an already existing edge
    fn add_edge_unchecked(&mut self, from: NodeId, to: NodeId) -> Result<bool, &'static str> {
        if from == to {
            return Err("cannot connect node to itself");
        }

        if let Some(nodefrom) = self.get_mut_node_from_id(from) {
            if matches!(
                nodefrom.kind,
                NodeKind::Drone {
                    pdr: _,
                    crashed: true
                }
            ) {
                return Err("cannot connect crashed drone");
            }
            nodefrom.adj.insert(to as NodeId);
        } else {
            panic!("cannot find noderepr for `from` node: #{from}")
        }

        if let Some(nodeto) = self.get_mut_node_from_id(to) {
            if matches!(
                nodeto.kind,
                NodeKind::Drone {
                    pdr: _,
                    crashed: true
                }
            ) {
                return Err("cannot connect crashed drone");
            }
            nodeto.adj.insert(from as NodeId);
        } else {
            panic!("cannot find noderepr for `to` node: #{to}")
        }

        let mut existed_already = false;
        if self.edges.contains_key(&(from, to)) || self.edges.contains_key(&(to, from)) {
            existed_already = true;
        }
        match from.cmp(&to) {
            std::cmp::Ordering::Less => {
                self.edges.insert((from, to), None);
            }
            std::cmp::Ordering::Greater => {
                self.edges.insert((to, from), None);
            }
            // node can't have edge that points to itself
            std::cmp::Ordering::Equal => unreachable!(),
        };

        Ok(existed_already)
    }

    /// updates existing edge, with the last packet that has traveled on it
    pub fn update_edge_activity(&mut self, from: NodeId, to: NodeId, packet_passed: PacketType) {
        match from.cmp(&to) {
            std::cmp::Ordering::Less => {
                if self.edges.contains_key(&(from, to)) {
                    self.edges
                        .insert((from, to), Some((packet_passed, Instant::now())));
                }
            }
            std::cmp::Ordering::Greater => {
                if self.edges.contains_key(&(to, from)) {
                    self.edges
                        .insert((to, from), Some((packet_passed, Instant::now())));
                }
            }
            // node can't have edge that points to itself
            std::cmp::Ordering::Equal => {}
        };
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
    pub fn crash_drone(&mut self, id: NodeId) -> Result<(), &'static str> {
        // ---------------------------------------------------------------
        // borrow the drone to change it to crashed, and save how it was before
        // ---------------------------------------------------------------
        let Some(drone) = self.nodes.iter_mut().find(|node| node.id == id) else {
            unreachable!("node to crash: #{id} not present in network")
        };
        let oldkind = drone.kind;
        drone.kind = match drone.kind {
            NodeKind::Drone {
                pdr,
                crashed: false,
            } => NodeKind::Drone { pdr, crashed: true },
            NodeKind::Drone { crashed: true, .. } => {
                return Err("trying to crash already crashed drone")
            }
            _ => unreachable!("trying to crash a node that is not a drone"),
        };

        // ---------------------------------------------------------------
        // borrow edges (drone went out of scope because its no longer used so you can borrow again) and filter out the edges that will be removed
        // ---------------------------------------------------------------
        let edges = self.edges.borrow_mut();
        let oldedges = edges.clone();

        edges.retain(|(from, to), _| *from != id && *to != id);

        // ---------------------------------------------------------------
        // check if the modified network is valid (can borrow because edges and drone have been dropped)
        // ---------------------------------------------------------------
        let res = self.is_valid();

        // ---------------------------------------------------------------
        // borrow one last time drone and edges to fix stuff back to how it was before in case
        // network is not valid after changes
        // ---------------------------------------------------------------
        match res {
            Ok(()) => Ok(()),
            Err(s) => {
                let Some(drone) = self.nodes.iter_mut().find(|node| node.id == id) else {
                    unreachable!("node to crash: #{id} not present in network")
                };
                drone.kind = oldkind;
                self.edges = oldedges;
                Err(s)
            }
        }
    }
}
