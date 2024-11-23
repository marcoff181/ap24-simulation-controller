use wg_2024::network::NodeId;

#[derive(Debug, Default)]
pub enum Screen {
    Start,
    #[default]
    Main,
    Move,
    AddNode,
    AddConnection {
        origin: NodeId,
    },
}
