pub const START_KEYS: [(&str, &str); 2] = [
    // ("↑", "Up"),
    // ("↓", "Down"),
    ("+", "Open initialization file"),
    ("q", "Quit"),
];

pub const MAIN_KEYS: [(&str, &str); 5] = [
    ("↑/↓", "Scroll list"),
    ("m", "Move node"),
    ("c", "Add connection"),
    ("+", "Add node"),
    ("q", "Quit"),
];

pub const MAIN_KEYS_OVER_DRONE: [(&str, &str); 7] = [
    ("↑/↓", "Scroll list"),
    ("m", "Move node"),
    ("c", "Add connection"),
    ("+", "Add node"),
    ("p", "Edit PDR"),
    ("k", "Crash"),
    ("q", "Quit"),
];

pub const MAIN_KEYS_ADD_CONNECTION: [(&str, &str); 3] = [
    ("↑/↓", "Scroll list"),
    ("Enter", "Connect to selected node"),
    ("q", "Quit"),
];

pub const MAIN_KEYS_ADD_NODE: [(&str, &str); 4] = [
    ("↑/↓/→/←", "Move"),
    ("s/c/d", "Set drone type"),
    ("Enter", "Add node"),
    ("q", "Quit"),
];

pub const MOVE_KEYS: [(&str, &str); 3] = [("↑/↓/→/←", "Move"), ("Enter", "Ok"), ("q", "Quit")];
