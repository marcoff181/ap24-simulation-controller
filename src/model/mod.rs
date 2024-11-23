use node_kind::NodeKind;
use node_representation::NodeRepresentation;
use ratatui::widgets::ListState;
use screen::Screen;
use wg_2024::config::Config;

pub mod node_kind;
pub mod node_representation;
pub mod screen;

#[derive(Debug, Default)]
pub struct Model {
    pub running: bool,
    pub screen: Screen,
    pub nodes: Vec<NodeRepresentation>,
    pub node_list_state: ListState,
}
impl Model{
    pub fn new(cfg: &Config) -> Self {
        let mut nodes: Vec<NodeRepresentation> = Vec::new();
    
        for d in cfg.drone.iter() {
            nodes.push(NodeRepresentation::new_from_cfgdrone(d));
        }
        for s in cfg.server.iter() {
            nodes.push(NodeRepresentation::new_from_cfgserver(s));
        }
        for c in cfg.client.iter() {
            nodes.push(NodeRepresentation::new_from_cfgclient(c));
        }
        Self {
            node_list_state: ListState::default(),
            running: false,
            screen: Screen::default(),
            nodes,
        }
    }

    // maybe should be in model?
    pub fn get_selected_kind(&self) -> Option<NodeKind> {
        let idx = self.node_list_state.selected()?;

        if idx < self.nodes.len() {
            Some(self.nodes[idx].kind)
        } else {
            None
        }
    }

    
    
}
