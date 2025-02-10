use crate::screen::{self};
use log::{debug, error, trace};
use messages::node_event::NodeEvent;
use screen::Window;

use wg_2024::{controller::DroneEvent, packet::PacketType};

impl crate::MySimulationController {
    pub(crate) fn save_nodeevent(&mut self, event: NodeEvent) {
        let Some(src) = event.source() else {
            error!("event has no source, caused by {:?}", event);
            return;
        };

        // -------------------------------------------------------------------------
        // update edge activity to see fragments being sent across the network
        // -------------------------------------------------------------------------
        if let NodeEvent::PacketSent(packet) = &event {
            if let Some(dst) = match &packet.pack_type {
                // when you send a flood request there is no information about who you sent it to,
                // so just to extract a bit of information we look at the last edge it traveled
                // across, and use that
                PacketType::FloodRequest(f) => {
                    if let Some(idx) = f.path_trace.len().checked_sub(2) {
                        if let Some((x, _)) = f.path_trace.get(idx) {
                            Some(*x)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => packet.routing_header.current_hop(),
            } {
                if self.network.edges.contains_key(&(src, dst))
                    || self.network.edges.contains_key(&(dst, src))
                {
                    self.network
                        .update_edge_activity(src, dst, packet.pack_type.clone());
                }
            };
        };

        // -------------------------------------------------------------------------
        // fix scrolling pushdown on certain tabs
        // -------------------------------------------------------------------------
        if let Window::Detail { tab } = self.screen.window {
            match event {
                NodeEvent::PacketSent(_) if tab == 0 && src == self.screen.focus => {
                    self.packet_table_state.scroll_down_by(1);
                }
                NodeEvent::StartingMessageTransmission(_)
                    if tab == 1 && src == self.screen.focus =>
                {
                    self.packet_table_state.scroll_down_by(1);
                }
                // message received behaves differently because we want to display on the
                // destionation node, not src
                NodeEvent::MessageReceived(ref message) if tab == 2 => {
                    if message.destination == self.screen.focus {
                        self.packet_table_state.scroll_down_by(1);
                    }
                }
                _ => {}
            }
        }

        // -------------------------------------------------------------------------
        // update each node with the received information of the event
        // -------------------------------------------------------------------------
        let node_opt = match event {
            NodeEvent::MessageReceived(ref m) => self.network.get_mut_node_from_id(m.destination),
            _ => self.network.get_mut_node_from_id(src),
        };
        if let Some(node) = node_opt {
            // save the data received in the correct location
            match event {
                NodeEvent::PacketSent(packet) => {
                    trace!("Client/Server #{src} sent event PacketSent with packet {packet}");
                    node.sent.push_front(packet.clone());
                }
                NodeEvent::MessageSentSuccessfully(message) => {
                    debug!(
                        "Client/Server #{src} sent event MessageSentSuccessfully with Message {:?}",
                        message
                    );
                    if node.msent.contains_key(&message.session_id) {
                        node.msent.insert(message.session_id, (message, true));
                    } else {
                        panic!("Got a MessageSentSuccessfully from #{src} with sid #{}, but didn't receive any StartingMessageTransmission for the same message yet",message.session_id)
                    }
                }
                NodeEvent::StartingMessageTransmission(message) => {
                    debug!(
                        "Client/Server #{src} sent event StartingMessageTransmission with Message {:?}",
                        message
                    );
                    node.msent.insert(message.session_id, (message, false));
                }
                NodeEvent::MessageReceived(message) => {
                    // message received behaves differently because we want to display on the
                    // destionation node, not src
                    let dst = message.destination;
                    debug!(
                        "Client/Server #{dst} sent event MessageReceived with Message {:?}",
                        message
                    );
                    node.mreceived.push_front(message);
                }
                NodeEvent::KnownNetworkGraph { source: _, graph } => {
                    debug!(
                        "Client/Server #{src} sent event KnownNetworkGraph with Network {:?}",
                        graph
                    );
                    node.knowntopology = graph;
                }
            };
        }
    }

    /// saves inside the `NodeRepresentation` the events received on the sc channel, logs the event
    /// received, and in the case of the Detail window, it scrolls the table state to match the
    /// newly added packet
    pub(crate) fn save_droneevent(&mut self, event: DroneEvent) {
        let packet = match event {
            DroneEvent::PacketSent(ref packet)
            | DroneEvent::PacketDropped(ref packet)
            | DroneEvent::ControllerShortcut(ref packet) => packet,
        };
        let id: u8 = match (&packet.pack_type, &event) {
            (PacketType::FloodRequest(flood_request), _) => {
                flood_request
                    .path_trace
                    .last()
                    .unwrap_or_else(|| panic!("path trace is empty, got {packet}"))
                    .0
            }
            (_, DroneEvent::PacketDropped(_)) => {
                packet.routing_header.current_hop().unwrap_or_else(|| {
                    panic!("could not find previous hop in packet {packet} for event {event:?}")
                })
            }

            _ => packet.routing_header.previous_hop().unwrap_or_else(|| {
                panic!("could not find previous hop in packet {packet} for event {event:?}")
            }),
        };

        if let DroneEvent::PacketSent(_) = event {
            if let Some(dst) = match &packet.pack_type {
                PacketType::FloodRequest(f) => {
                    if let Some(idx) = f.path_trace.len().checked_sub(2) {
                        if let Some((x, _)) = f.path_trace.get(idx) {
                            Some(*x)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => packet.routing_header.current_hop(),
            } {
                if self.network.edges.contains_key(&(id, dst))
                    || self.network.edges.contains_key(&(dst, id))
                {
                    self.network
                        .update_edge_activity(id, dst, packet.pack_type.clone());
                }
            };
        };

        if let Some(node) = self.network.get_mut_node_from_id(id) {
            match event {
                DroneEvent::PacketSent(packet) => {
                    trace!("Drone {id} sent event PacketSent with packet {packet}");
                    if let PacketType::MsgFragment(_) = packet.pack_type {
                        node.n_frags_sent = node.n_frags_sent.saturating_add(1);
                    }
                    node.sent.push_front(packet);
                    if let Window::Detail { tab: 0 } = self.screen.window {
                        self.packet_table_state.scroll_down_by(1);
                    };
                }
                DroneEvent::PacketDropped(packet) => {
                    trace!("Drone {id} sent event PacketDropped with packet {packet}");
                    node.n_frags_dropped = node.n_frags_dropped.saturating_add(1);
                    node.n_frags_sent = node.n_frags_sent.saturating_add(1);
                    node.dropped.push_front(packet);
                    if let Window::Detail { tab: 1 } = self.screen.window {
                        self.packet_table_state.scroll_down_by(1);
                    };
                }
                DroneEvent::ControllerShortcut(packet) => {
                    debug!("Drone {id} sent event ControllerShortcut with packet {packet}");
                    node.shortcutted.push_front(packet);
                    if let Window::Detail { tab: 2 } = self.screen.window {
                        self.packet_table_state.scroll_down_by(1);
                    };
                }
            }
        }
    }
}
