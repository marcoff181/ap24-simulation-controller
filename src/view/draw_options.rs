use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    hash::Hash,
};

use messages::node_event::EventNetworkGraph;
use rand::random_range;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled, Stylize},
    symbols::{self, Marker},
    widgets::{
        canvas::{Canvas, Context, Line},
        Block, Borders, Padding, Widget,
    },
};
use wg_2024::network::NodeId;

use crate::{
    network::{node_kind::NodeKind, node_representation::NodeRepresentation},
    screen::{Screen, Window},
    utilities::theme::*,
    Network,
};

pub struct DrawGraphOptions {
    pub padding: f64,
    pub lines_back: HashSet<(NodeId, NodeId)>,
    pub lines_front: HashSet<(NodeId, NodeId)>,
    pub back_color: Color,
    pub front_color: Color,
    pub nodes: HashMap<NodeId, DrawNodeOptions>,
}

pub struct DrawNodeOptions {
    pub x: f64,
    pub y: f64,
    pub style: Style,
    pub label: String,
}
impl DrawGraphOptions {
    pub fn from_noderepr(n: &NodeRepresentation) -> Self {
        let mut lines_front = HashSet::new();
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
            lines_front.insert((n.id, *k));
        }

        DrawGraphOptions {
            padding: 30.0,
            lines_back: HashSet::new(),
            lines_front,
            back_color: TEXT_COLOR,
            front_color: TEXT_COLOR,
            nodes,
        }
    }
    pub fn from_topology(top: &EventNetworkGraph) -> Self {
        let lines_back = HashSet::new();
        let mut lines_front = HashSet::new();
        let back_color = TEXT_COLOR;
        let front_color = TEXT_COLOR;
        let mut nodes = HashMap::new();

        let mut x = 0.0;

        for n in top.nodes.iter() {
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
                    x,
                    y: random_range(1..5) as f64,
                    style,
                    label,
                },
            );
            x += 10.0;

            for nghb in n.neighbors.iter() {
                lines_front.insert((id, *nghb));
            }
        }

        DrawGraphOptions {
            padding: 0.0,
            lines_front,
            lines_back,
            back_color,
            front_color,
            nodes,
        }
    }

    pub fn from_network(network: &Network, screen: &Screen) -> Self {
        let mut lines_back: HashSet<(NodeId, NodeId)> = HashSet::new();
        let mut lines_front: HashSet<(NodeId, NodeId)> = HashSet::new();
        let back_color = TEXT_COLOR;
        let mut nodes: HashMap<NodeId, DrawNodeOptions> = HashMap::new();

        let id = screen.focus;

        let front_color = match screen.window {
            Window::AddConnection { origin } => ADD_EDGE_COLOR,
            Window::ChangePdr { pdr } => todo!(),
            Window::Detail { tab } => todo!(),
            Window::Error { message } => todo!(),
            Window::Main => HIGHLIGHT_COLOR,
            Window::Move => ADD_EDGE_COLOR,
        };

        for (from, to) in network.edges.iter() {
            if *from == id || *to == id {
                match screen.window {
                    Window::Main | Window::Move => {
                        lines_front.insert((*from, *to));
                    }
                    Window::AddConnection { origin } => {
                        if origin == *from || origin == *to {
                            lines_front.insert((*from, *to));
                        } else {
                            lines_back.insert((*from, *to));
                        }
                    }
                    _ => unreachable!(),
                }
            } else {
                lines_back.insert((*from, *to));
            };
        }
        for n in network.nodes.iter() {
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
                    x: n.x as f64,
                    y: n.y as f64,
                    style,
                    label: n.short_label().to_string(),
                },
            );
        }

        DrawGraphOptions {
            padding: 0.0,
            lines_front,
            lines_back,
            back_color,
            front_color,
            nodes,
        }
    }
}
