#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeKind {
    Drone { pdr: f32, crashed: bool },
    Client,
    Server,
}
