use std::collections::HashSet;

use node_kind::NodeKind;
use node_representation::NodeRepresentation;
use ratatui::widgets::ListState;
use screen::Screen;
use wg_2024::{config::Config, network::NodeId};

pub mod node_kind;
pub mod node_representation;
pub mod screen;

#[derive(Debug, Default)]
pub struct Model {
    //todo: some of these don't need to be public
    pub screen: Screen,
    pub nodes: Vec<NodeRepresentation>,
    pub edges: HashSet<(NodeId, NodeId)>,
    pub node_list_state: ListState,
}
impl Model {
    pub fn new(cfg: &Config) -> Self {
        let nodes: Vec<NodeRepresentation> = Vec::new();
        let edges: HashSet<(NodeId, NodeId)> = HashSet::new();

        let mut model = Self {
            node_list_state: ListState::default(),
            screen: Screen::default(),
            nodes,
            edges,
        };

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

    /// adds a default node to the model and selects it
    pub fn spawn_default_node(&mut self) {
        // todo find a way to not risk it being a duplicate id
        self.nodes.push(NodeRepresentation::default());
        self.node_list_state.select_last();
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        match from.cmp(&to) {
            std::cmp::Ordering::Less => {
                self.edges.insert((from, to));
            }
            std::cmp::Ordering::Equal => {
                self.edges.insert((to, from));
            }
            // node can't have edge that points to itself
            std::cmp::Ordering::Greater => {}
        };

        // todo: decide if there is need to keep this logic related to adj
        if let Some(nodefrom_id) = self.get_mut_node_from_id(from) {
            nodefrom_id.adj.insert(to as NodeId);
        }

        if let Some(nodeto_id) = self.get_mut_node_from_id(to) {
            nodeto_id.adj.insert(from as NodeId);
        }
    }

    pub fn get_selected_kind(&self) -> Option<NodeKind> {
        let idx = self.node_list_state.selected()?;

        if idx < self.nodes.len() {
            Some(self.nodes[idx].kind)
        } else {
            None
        }
    }

    pub fn selected_node_id(&self) -> Option<NodeId> {
        let idx = self.node_list_state.selected()?;

        if idx < self.nodes.len() {
            Some(self.nodes[idx].id)
        } else {
            None
        }
    }

    pub fn get_selected_node(&self) -> Option<&NodeRepresentation> {
        let idx = self.node_list_state.selected()?;

        if idx < self.nodes.len() {
            Some(&self.nodes[idx])
        } else {
            None
        }
    }

    pub fn get_mut_selected_node(&mut self) -> Option<&mut NodeRepresentation> {
        let idx = self.node_list_state.selected()?;

        if idx < self.nodes.len() {
            Some(&mut self.nodes[idx])
        } else {
            None
        }
    }

    pub fn get_node_from_id(&self, id: NodeId) -> Option<&NodeRepresentation> {
        self.nodes.iter().find(|&node| node.id == id)
    }

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

        self.edges.retain(|(from, to)| *from != id && *to != id);
    }

    /// selects the node with the given 'id' (if there is one)
    pub fn select_node(&mut self, id: NodeId) {
        if let Some(pos) = self.nodes.iter().position(|n| n.id == id) {
            self.node_list_state.select(Some(pos));
        }
    }
}
