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
