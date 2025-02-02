pub const MAIN_KEYS: [(&str, &str); 6] = [
    ("↑/↓", "Scroll list"),
    ("m", "Move node"),
    ("c", "Add connection"),
    ("+", "Spawn node"),
    ("q", "Quit"),
    ("d", "Detail view"),
];

pub const ERROR_KEYS: [(&str, &str); 2] = [("Enter", "Ok"), ("q", "Quit")];

pub const MAIN_KEYS_ADD_CONNECTION: [(&str, &str); 3] = [
    ("↑/↓", "Scroll list"),
    ("Enter", "Connect to selected node"),
    ("q", "Quit"),
];

pub const MOVE_KEYS: [(&str, &str); 3] = [("↑/↓/→/←", "Move"), ("Enter", "Ok"), ("q", "Quit")];

pub const DETAIL_KEYS_DRONE: [(&str, &str); 6] = [
    ("↑/↓", "Scroll list"),
    ("tab", "Next list"),
    ("p", "Edit PDR"),
    ("k", "Crash"),
    ("Enter", "Done"),
    ("q", "Quit"),
];

pub const PDR_KEYS: [(&str, &str); 3] = [
    ("↑/↓", "Increase/decrease PDR"),
    ("Enter", "Done"),
    ("q", "Quit"),
];

pub const DETAIL_KEYS_NOTDRONE: [(&str, &str); 4] = [
    ("↑/↓", "Scroll list"),
    ("tab", "Next list"),
    ("Enter", "Done"),
    ("q", "Quit"),
];
