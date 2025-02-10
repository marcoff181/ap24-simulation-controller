use std::{collections::HashMap, time::Instant};

use messages::node_event::EventNetworkGraph;
use ratatui::style::{Color, Style, Stylize};
use wg_2024::{network::NodeId, packet::PacketType};

use crate::{
    network::node_representation::NodeRepresentation,
    screen::{Screen, Window},
    utilities::theme::{
        ADD_EDGE_COLOR, BG_COLOR, CLIENT_COLOR, DRONE_COLOR, HIGHLIGHT_COLOR, PACKET_ACK_COLOR,
        PACKET_FLOOD_REQUEST_COLOR, PACKET_FLOOD_RESPONSE_COLOR, PACKET_FRAGMENT_COLOR,
        PACKET_NACK_COLOR, SERVER_COLOR, TEXT_COLOR,
    },
    Network,
};

pub struct DrawGraphOptions {
    pub padding: f64,
    pub lines_back: HashMap<(NodeId, NodeId), Color>,
    pub lines_front: HashMap<(NodeId, NodeId), Color>,
    pub nodes: HashMap<NodeId, DrawNodeOptions>,
}

pub struct DrawNodeOptions {
    pub x: f64,
    pub y: f64,
    pub style: Style,
    pub label: String,
}

fn active_edge_color(x: &Option<(PacketType, Instant)>) -> Color {
    if let Some((t, inst)) = x {
        let d = Instant::now().saturating_duration_since(*inst);
        if d.as_secs() < 3 {
            match t {
                PacketType::MsgFragment(_) => PACKET_FRAGMENT_COLOR,
                PacketType::Ack(_) => PACKET_ACK_COLOR,
                PacketType::Nack(_) => PACKET_NACK_COLOR,
                PacketType::FloodRequest(_) => PACKET_FLOOD_REQUEST_COLOR,
                PacketType::FloodResponse(_) => PACKET_FLOOD_RESPONSE_COLOR,
            }
        } else {
            TEXT_COLOR
        }
    } else {
        TEXT_COLOR
    }
}
impl DrawGraphOptions {
    pub fn from_noderepr(n: &NodeRepresentation) -> Self {
        let mut lines_front = HashMap::new();
        let mut nodes: HashMap<u8, DrawNodeOptions> = HashMap::new();
        nodes.insert(
            n.id,
            DrawNodeOptions {
                x: 0.0,
                y: 0.0,
                style: Style::default().bg(n.color()),
                label: n.short_label(),
            },
        );

        let count = n.adj.len();
        for (i, k) in n.adj.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / count as f64;
            let x = 100.0 * angle.cos();
            let y = 100.0 * angle.sin();

            nodes.insert(
                *k,
                DrawNodeOptions {
                    x,
                    y,
                    style: Style::default().reversed(),
                    label: format!("(#{k})"),
                },
            );
            lines_front.insert((n.id, *k), TEXT_COLOR);
        }

        DrawGraphOptions {
            padding: 30.0,
            lines_back: HashMap::new(),
            lines_front,
            nodes,
        }
    }
    pub fn from_topology(top: &EventNetworkGraph) -> Self {
        let lines_back = HashMap::new();
        let mut lines_front = HashMap::new();
        let mut nodes = HashMap::new();

        for n in &top.nodes {
            let style;
            let label;

            match n.node_type {
                wg_2024::packet::NodeType::Client => {
                    label = "(C)".to_owned();
                    style = Style::new().bg(CLIENT_COLOR);
                }
                wg_2024::packet::NodeType::Drone => {
                    label = "(D)".to_owned();
                    style = Style::new().bg(DRONE_COLOR);
                }
                wg_2024::packet::NodeType::Server => {
                    label = "(S)".to_owned();
                    style = Style::new().bg(SERVER_COLOR);
                }
            }

            let id = n.node_id;
            nodes.insert(
                id,
                DrawNodeOptions {
                    x: f64::from(id) * 5.,
                    y: f64::from(id) % 3. * 5.,
                    style,
                    label,
                },
            );

            for nghb in &n.neighbors {
                lines_front.insert((id, *nghb), TEXT_COLOR);
            }
        }

        DrawGraphOptions {
            padding: 0.0,
            lines_front,
            lines_back,
            nodes,
        }
    }

    pub fn from_network(network: &Network, screen: &Screen) -> Self {
        let mut lines_back = HashMap::new();
        let mut lines_front = HashMap::new();
        let mut nodes: HashMap<NodeId, DrawNodeOptions> = HashMap::new();

        let id = screen.focus;

        let front_color = match screen.window {
            Window::AddConnection { .. } | Window::Move => ADD_EDGE_COLOR,
            Window::Main => TEXT_COLOR,
            _ => unreachable!(),
        };

        // add one single line between nodes that are being connected
        if let Window::AddConnection { origin } = screen.window {
            if origin != screen.focus {
                lines_front.insert((screen.focus, origin), front_color);
            }
        }

        for ((from, to), x) in &network.edges {
            if *from == id || *to == id {
                match screen.window {
                    Window::Main | Window::Move => {
                        lines_front.insert((*from, *to), active_edge_color(x));
                    }
                    Window::AddConnection { .. } => {
                        lines_back.insert((*from, *to), active_edge_color(x));
                    }
                    _ => unreachable!(),
                }
            } else {
                lines_back.insert((*from, *to), active_edge_color(x));
            };
        }
        for n in &network.nodes {
            // special coloring
            let selected_index = screen.focus;
            let mut style = Style::default();
            match screen.window {
                // highlight selected node
                Window::Main | Window::Move => {
                    if selected_index == n.id {
                        style = style.bg(HIGHLIGHT_COLOR);
                        style = style.fg(BG_COLOR);
                        style = style.bold();
                    } else {
                        style = style.bg(n.color());
                        style = style.fg(TEXT_COLOR);
                    }
                }
                // highlight node from which connection starts
                // and highlight green selected node for destination
                Window::AddConnection { origin } => {
                    if n.id == origin {
                        style = style.bg(HIGHLIGHT_COLOR);
                        style = style.fg(BG_COLOR);
                        style = style.bold();
                    } else if selected_index == n.id {
                        style = style.bg(Color::Green);
                        style = style.fg(TEXT_COLOR);
                        style = style.bold();
                    } else {
                        style = style.bg(n.color());
                        style = style.fg(TEXT_COLOR);
                    }
                }
                _ => unreachable!(),
            }
            nodes.insert(
                n.id,
                DrawNodeOptions {
                    x: f64::from(n.x),
                    y: f64::from(n.y),
                    style,
                    label: n.short_label().to_string(),
                },
            );
        }

        DrawGraphOptions {
            padding: 0.0,
            lines_front,
            lines_back,
            nodes,
        }
    }
}
