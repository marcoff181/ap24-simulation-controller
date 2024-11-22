#[derive(Debug, Default)]
pub enum Screen {
    Start,
    #[default]
    Main,
    Move,
    AddNode,
    AddConnection {
        origin: u32,
    },
}
