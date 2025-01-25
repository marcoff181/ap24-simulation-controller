use std::collections::HashSet;

use messages::node_event::{EventNetworkGraph, EventNetworkNode};
use ratatui::prelude::*;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::{
    Block, Borders, Clear, Gauge, ListState, Padding, Paragraph, TableState, Wrap,
};
use symbols::Marker;
//use wg_2024::config::{Client, Drone, Server};

use crate::network::node_kind::NodeKind;
use crate::network::node_representation::NodeRepresentation;
use crate::network::Network;
use crate::screen::Window;
use crate::utilities::theme::{
    BG_COLOR, CLIENT_COLOR, CRASH_COLOR, DRONE_COLOR, HIGHLIGHT_COLOR, SERVER_COLOR, TEXT_COLOR,
};
use crate::Screen;

use super::draw_options::DrawGraphOptions;
use super::simulation::render_simulation;

pub fn node_detail(node: &NodeRepresentation, area: Rect, frame: &mut Frame) {
    let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);
    let mut header = Line::default();
    match node.kind {
        NodeKind::Drone { pdr, crashed } => {
            header.push_span(Span::from("Drone").style(Style::default().bg(DRONE_COLOR)));
        }
        NodeKind::Client => {
            header.push_span(Span::from("Client").style(Style::default().bg(CLIENT_COLOR)));
        }
        NodeKind::Server => {
            header.push_span(Span::from("Server").style(Style::default().bg(SERVER_COLOR)));
        }
    }
    header.push_span(Span::from(format!(" #{}", node.id)));

    header.render(left, frame.buffer_mut());

    match node.kind {
        NodeKind::Drone { pdr, crashed } => {
            render_simulation(
                DrawGraphOptions::from_noderepr(node),
                right,
                frame.buffer_mut(),
            );
        }
        NodeKind::Client | NodeKind::Server => {
            render_simulation(
                DrawGraphOptions::from_topology(&node.knowntopology),
                right,
                frame.buffer_mut(),
            );
        }
    }
}
