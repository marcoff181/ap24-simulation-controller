use crate::network::{self};
use crossbeam_channel::unbounded;
use log::{debug, error};
use network::{node_kind::NodeKind, node_representation::NodeRepresentation};
use std::{
    collections::{HashMap, HashSet},
    thread::Builder,
};

use wg_2024::{controller::DroneCommand, drone::Drone, network::NodeId, packet::Packet};

impl crate::MySimulationController {
    /// sends the given packet directly to its final destination
    /// Panics
    /// - when the packet has no destination
    /// - if there is no packet sender for the destination node
    pub(crate) fn shortcut_packet(&mut self, mut packet: Packet) {
        let dst = packet
            .routing_header
            .destination()
            .unwrap_or_else(|| panic!("Destination for packet {packet} not found"));
        if !packet.routing_header.hops.is_empty() {
            packet.routing_header.hop_index = packet.routing_header.hops.len() - 1;
        }
        let sender = self
            .packet_send
            .get(&dst)
            .unwrap_or_else(|| panic!("packet sender for #{dst} not found"));
        debug!("Shortcutted packet: {packet}");
        let _ = sender.send(packet);
    }

    /// adds a connection between two nodes, first checking that the given source and destination
    /// follow certain rules(connections does not exist, at least one is a drone, not between same
    /// node,none of them crashed), then sends to the corresponding nodes in the simulation the command to add a
    /// neighbor, if unsuccessful returns an error with explanation, when the error is unexpected
    /// it panics
    pub(crate) fn add_connection(&mut self, from: NodeId, to: NodeId) -> Result<(), &'static str> {
        debug!("checking if connection to be added is not between two client/server, not between same node, does not exist...");

        match self.network.add_edge(from, to) {
            Ok(()) => {
                // tell the real nodes via command channels to add edge
                debug!("getting command and packet senders to tell nodes to add neighbor...");
                if let (
                    Some(command_sender_from),
                    Some(command_sender_to),
                    Some(packet_sender_to),
                    Some(packet_sender_from),
                ) = (
                    self.command_send.get(&from),
                    self.command_send.get(&to),
                    self.packet_send.get(&to),
                    self.packet_send.get(&from),
                ) {
                    let _ = command_sender_from
                        .send(DroneCommand::AddSender(to, packet_sender_to.clone()));
                    let _ = command_sender_to
                        .send(DroneCommand::AddSender(from, packet_sender_from.clone()));

                    debug!("succesfully sent addSender commands to neighbors...");
                    Ok(())
                } else {
                    unreachable!(
                        "could not find command senders or packet senders for nodes with id {} and {}\npacket senders: {:?}\ncommand senders: {:?}",
                        from, to, self.packet_send,self.command_send
                    );
                }
            }
            Err(s) => {
                debug!("add_edge function returned error: {s}");
                Err(s)
            }
        }
    }

    /// send crash command to drone, removesender command to neighbors, remove the sender kept by
    /// the sc
    /// # Panics
    /// - if there is no noderepresentation in the network for the crashing drone
    /// - if the id is not of a drone
    /// - if there is no command sender for the drone or any of its neighbors
    /// - if there is no packet sender for the drone
    pub(crate) fn crash(&mut self, id: NodeId) -> Result<(), &'static str> {
        // check that drone can be removed
        self.network.crash_drone(id)?;

        if let Some(drone_command_sender) = self.command_send.get(&id) {
            // send command to corresponding drone to crash
            let _ = drone_command_sender.send(DroneCommand::Crash);
            let node = self
                .network
                .get_node_from_id(id)
                .expect("could not find noderepresentation for drone {id}");
            if !matches!(node.kind, NodeKind::Drone { .. }) {
                unreachable!("trying to crash non-drone node #{id}")
            }
            let nodes = node.adj.clone();
            for n in nodes {
                let sender = self
                    .command_send
                    .get(&n)
                    .expect("could not find comm sender for drone {n}");

                // send command to neighbor drones to remove sender
                let _ = sender.send(DroneCommand::RemoveSender(id));

                if let Some(x) = self.network.get_mut_node_from_id(id) {
                    x.adj.remove(&n);
                }
                if let Some(x) = self.network.get_mut_node_from_id(n) {
                    x.adj.remove(&id);
                }
            }
            // remove the sender kept by the sc
            self.packet_send
                .remove(&id)
                .expect("could not find packet sender for drone {id}");

            Ok(())
        } else {
            panic!("could not find command sender for drone #{id}")
        }
    }

    /// changes pdr of drone in representation and of actual drone
    /// Panics
    /// panics if it can't find drone in the network, if it's not a drone, if it's crashed, if
    /// there is no command sender for it
    pub(crate) fn change_pdr(&mut self, newpdr: f32) {
        let node = self
            .network
            .get_mut_node_from_id(self.screen.focus)
            .expect("could not find drone with matching id");
        match node.kind {
            NodeKind::Drone {
                ref mut pdr,
                crashed: false,
            } => {
                // change pdr of simulation drone
                *pdr = newpdr;

                // change pdr of actual drone
                let command_send = self
                    .command_send
                    .get(&node.id)
                    .expect("could not find command sender for drone");
                let _ = command_send.send(DroneCommand::SetPacketDropRate(newpdr));
            }
            _ => unreachable!("either not drone or crashed"),
        }
    }

    /// creates drone with random id, spawns its thread, and adds it to the SC
    /// # Panics
    /// panics if it can't create the drone thread
    pub(crate) fn spawn_drone(&mut self) {
        let kind = NodeKind::Drone {
            pdr: 0.05,
            crashed: false,
        };
        let id = self.random_unique_id();
        let name = format!("SkyLink#{id}");
        let mut n = NodeRepresentation::new(id, 0, 0, kind, HashSet::new());
        n.thread_name.clone_from(&name);

        let event_send = self.droneevent_send.clone();
        let (command_send, command_recv) = unbounded::<DroneCommand>();
        let (packet_send, packet_recv) = unbounded::<Packet>();

        self.command_send.insert(n.id, command_send);
        self.packet_send.insert(n.id, packet_send);

        let handle = Builder::new()
            .name(name)
            .spawn(move || {
                skylink::SkyLinkDrone::new(
                    n.id,
                    event_send,
                    command_recv,
                    packet_recv,
                    HashMap::new(),
                    0.05,
                )
                .run();
            })
            .expect("could not spawn drone thread");

        self.node_handles.insert(n.id, handle);

        self.network.nodes.push(n.clone());

        self.node_list_state.select_last();
        self.screen.focus = n.id;
        self.screen.kind = kind;
    }
}
