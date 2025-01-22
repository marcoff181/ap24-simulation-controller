#[derive(Debug)]
pub enum AppMessage {
    // used in move node and add node
    MoveNode { x: i8, y: i8 },

    // used in Detail
    ChangeTab,

    // used in main
    WindowAddConnection,
    WindowChangePDR,
    WindowMove,
    WindowDetail,
    SpawnDrone,
    Crash,

    // used in main, add connection
    ScrollUp,
    ScrollDown,

    // used in add connection, add node, Detail, Move, Changepdr
    Done,

    // used in all
    Quit,
}
