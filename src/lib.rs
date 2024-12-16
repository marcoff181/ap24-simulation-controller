mod keypress_handler;
mod model;
mod utilities;
mod view;

use std::{collections::HashMap, thread::JoinHandle};

use crate::model::Model;
use crossbeam_channel::{Receiver, Sender};
use log::debug;
use model::{node_kind::NodeKind, screen::Screen};
use ratatui::DefaultTerminal;
use utilities::app_message::AppMessage;
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::Packet,
};

pub struct SimControllerOptions {
    pub command_send: HashMap<NodeId, Sender<DroneCommand>>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub event_recv: Receiver<DroneEvent>,
    pub config: Config,
    pub node_handles: Vec<JoinHandle<()>>,
}

pub struct MySimulationController {
    command_send: HashMap<NodeId, Sender<DroneCommand>>,
    command_recv: Receiver<DroneEvent>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    config: Config,
    model: Model,
    pub node_handles: Vec<JoinHandle<()>>,
}

impl MySimulationController {
    pub fn new(opt: SimControllerOptions) -> Self {
        MySimulationController {
            command_send: opt.command_send,
            command_recv: opt.event_recv,
            packet_send: opt.packet_send,
            config: opt.config.clone(),
            model: Model::new(&opt.config),
            node_handles: opt.node_handles,
        }
    }

    // could return Result and then thread handler in network intializer handles the Error
    pub fn run(&mut self) {
        let terminal = ratatui::init();
        let _result = self.start(terminal);
        ratatui::restore();
    }
}

impl MySimulationController {
    fn start(&mut self, mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
        let mut running = true;
        self.model.node_list_state.select(Some(0));

        while running {
            terminal.draw(|frame| crate::view::render(&mut self.model, frame))?;

            if let Some(message) = keypress_handler::handle_crossterm_events(&mut self.model)? {
                match message {
                    AppMessage::AddConnection { from, to } => self.add_connection(from, to),
                    AppMessage::Crash { drone: id } => self.crash(id),
                    AppMessage::Quit => running = false,
                    AppMessage::AddNode { node } => todo!(),
                }
            };

            while let Ok(event) = self.command_recv.try_recv() {
                match event {
                    DroneEvent::PacketSent(packet) => self.save_packet_sent(packet),
                    DroneEvent::PacketDropped(packet) => self.save_packet_dropped(packet),
                    DroneEvent::ControllerShortcut(packet) => todo!(),
                }
            }
        }

        Ok(())
    }
    // handle commands from drone

    fn save_packet_sent(&mut self, packet: Packet) {
        let id = packet.routing_header.hops[packet.routing_header.hop_index - 1];
        if let Some(node) = self.model.get_mut_node_from_id(id) {
            debug!("Drone {id} sent event PacketSent with packet {packet}");
            node.sent.push_front(packet);
        }
    }

    fn save_packet_dropped(&mut self, packet: Packet) {
        let id = packet.routing_header.hops[packet.routing_header.hop_index - 1];
        if let Some(node) = self.model.get_mut_node_from_id(id) {
            node.dropped.push_front(packet);
        }
    }

    fn add_connection(&mut self, from: NodeId, to: NodeId) {
        //check connection is not between two clients/servers
        if let (Some(nfrom), Some(nto)) = (
            self.model.get_node_from_id(from),
            self.model.get_node_from_id(to),
        ) {
            if !matches!(nfrom.kind, NodeKind::Drone { .. })
                && !matches!(nto.kind, NodeKind::Drone { .. })
            {
                panic!(
                    "Cannot connect {} and {}, at least one should be a drone",
                    nfrom.kind, nto.kind
                )
            }
        } else {
            panic!("nodes not found");
        }

        // tell the real nodes via command channels to add edge
        if let (
            Some(command_sender_from),
            Some(command_sender_to),
            Some(packet_sender_to),
            Some(packet_sender_from),
        ) = (
            self.command_send.get(&from),
            self.command_send.get(&to),
            self.packet_send.get(&from),
            self.packet_send.get(&to),
        ) {
            command_sender_from.send(DroneCommand::AddSender(to, packet_sender_to.clone()));
            command_sender_to.send(DroneCommand::AddSender(from, packet_sender_from.clone()));

            // for now we assume they succesfully added channel, and show it in the model
            self.model.add_edge(from, to);
            self.model.select_node(from);
            self.model.screen = Screen::Main;
        } else {
            panic!("could not create connection")
        }
    }

    fn crash(&mut self, id: NodeId) {
        // send command to corresponding drone to crash
        if let Some(drone_command_sender) = self.command_send.get(&id) {
            // todo: handle error
            let _ = drone_command_sender.send(DroneCommand::Crash);
        }

        // todo: do we need to tell the other drones to remove edges that point to crashed drone?
        // wg will decide but I think drones should be the ones to handle the crash

        // set in the model the corresponding node to crashed true
        self.model.crash_drone(id);
    }

    /// adds `to the proper simulation` a node that has already been added to the model
    fn add_node(&mut self, id: NodeId) {
        if let Some(n) = self.model.get_node_from_id(id) {
            match n.kind {
                NodeKind::Drone { pdr, crashed } => todo!(),
                NodeKind::Client => todo!(),
                NodeKind::Server => todo!(),
            }
        } else {
            //todo:improve
            panic!("added drone not found");
        }
    }
}
