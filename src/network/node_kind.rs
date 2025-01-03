use std::fmt::Display;

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum NodeKind {
    // ?could be used on WG
    Drone {
        pdr: f32,
        crashed: bool,
    },
    // todo: don't do this
    #[default]
    Client,
    Server,
}

impl Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Drone { pdr: _, crashed: _ } => write!(f, "Drone"),
            NodeKind::Client => write!(f, "Client"),
            NodeKind::Server => write!(f, "Server"),
        }
    }
}
