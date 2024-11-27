use ratatui::style::Color;

pub const BG_COLOR: Color = Color::Black;
pub const BOTTOMPANEL_BG: Color = Color::Black;
pub const TEXT_COLOR: Color = Color::White;
pub const INVERTED_TEXT_COLOR: Color = Color::Black;
pub const HIGHLIGHT_COLOR: Color = Color::LightYellow;
pub const DRONE_COLOR: Color = Color::LightBlue;
pub const SERVER_COLOR: Color = Color::LightMagenta;
pub const CLIENT_COLOR: Color = Color::Cyan;
pub const CRASH_COLOR: Color = Color::Red;
pub const ADD_EDGE_COLOR: Color = Color::Green;

pub const PACKET_NACK_COLOR: Color = CRASH_COLOR;
pub const PACKET_ACK_COLOR: Color = ADD_EDGE_COLOR;
pub const PACKET_FRAGMENT_COLOR: Color = Color::LightBlue;
pub const PACKET_FLOOD_REQUEST_COLOR: Color = Color::LightMagenta;
pub const PACKET_FLOOD_RESPONSE_COLOR: Color = Color::LightYellow;
