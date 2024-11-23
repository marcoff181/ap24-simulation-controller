use std::collections::HashSet;

use node_kind::NodeKind;
use node_representation::NodeRepresentation;
use ratatui::widgets::ListState;
use screen::Screen;
use wg_2024::{
    config::Config,
    network::NodeId,
};

pub mod node_kind;
pub mod node_representation;
pub mod screen;

#[derive(Debug, Default)]
pub struct Model {
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

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        if from < to {
            self.edges.insert((from, to));
        } else if to < from {
            self.edges.insert((to, from));
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

    pub fn selected_node(&self) -> Option<&NodeRepresentation> {
        let idx = self.node_list_state.selected()?;

        if idx < self.nodes.len() {
            Some(&self.nodes[idx])
        } else {
            None
        }
    }

    pub fn get_node_from_id(&self, id:NodeId)-> Option<&NodeRepresentation>{
        for node in self.nodes.iter(){
            if node.id == id {
                return Some(&node);
            }
        }
        None
    }
}
