
use ratatui::prelude::*;
//use wg_2024::config::{Client, Drone, Server};

use crate::network::node_kind::NodeKind;
use crate::network::node_representation::NodeRepresentation;
use crate::utilities::theme::{
    CLIENT_COLOR, DRONE_COLOR, SERVER_COLOR,
};

use super::draw_options::DrawGraphOptions;
use super::simulation::render_simulation;

pub fn node_detail(node: &NodeRepresentation, area: Rect, frame: &mut Frame) {
    let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);
    let mut header = Line::default();
    let nameline = Line::from(format!("Thread name: {}", node.thread_name));
    let mut content = Text::default();
    match node.kind {
        NodeKind::Drone { pdr, crashed } => {
            header.push_span(Span::from("Drone").style(Style::default().bg(DRONE_COLOR)));
            header.push_span(Span::from(format!(" #{}", node.id)));
            content.push_line(header);
            content.push_line(nameline);
            content.push_line(format!("crashed:{crashed}"));
            content.push_line(format!("config pdr:{pdr}"));
            content.push_line(format!(
                "actual pdr:{:.3}",
                (node.n_frags_dropped as f64) / (node.n_frags_sent as f64),
            ));
            content.push_line(format!(
                "dropped:{}/{} fragments",
                node.n_frags_dropped, node.n_frags_sent
            ));
        }
        NodeKind::Client => {
            header.push_span(Span::from("Client").style(Style::default().bg(CLIENT_COLOR)));
            header.push_span(Span::from(format!(" #{}", node.id)));
            content.push_line(header);
            content.push_line(nameline);
        }
        NodeKind::Server => {
            header.push_span(Span::from("Server").style(Style::default().bg(SERVER_COLOR)));
            header.push_span(Span::from(format!(" #{}", node.id)));
            content.push_line(header);
            content.push_line(nameline);
        }
    }

    content.render(left, frame.buffer_mut());

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
