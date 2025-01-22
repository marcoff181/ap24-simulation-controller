mod keypress_handler;
mod network;
mod screen;
mod utilities;
mod view;

use crate::screen::Screen;
use std::{collections::HashMap, thread::JoinHandle};

use crate::network::Network;
use crossbeam_channel::{Receiver, Sender};
use log::debug;
use network::{node_kind::NodeKind, node_representation::NodeRepresentation};
use ratatui::{
    widgets::{ListState, TableState},
    DefaultTerminal,
};
use screen::Window;
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
    // external comms
    command_send: HashMap<NodeId, Sender<DroneCommand>>,
    command_recv: Receiver<DroneEvent>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    node_handles: Vec<JoinHandle<()>>,
    // internal state
    running: bool,
    network: Network,
    node_list_state: ListState,
    packet_table_state: TableState,
    screen: Screen,
}

impl MySimulationController {
    pub fn new(opt: SimControllerOptions) -> Self {
        MySimulationController {
            command_send: opt.command_send,
            command_recv: opt.event_recv,
            packet_send: opt.packet_send,
            node_handles: opt.node_handles,
            network: Network::new(&opt.config),
            node_list_state: ListState::default().with_selected(Some(0)),
            packet_table_state: TableState::default().with_selected(0),
            screen: Screen {
                focus: opt.config.drone[0].id,
                //TODO: fix
                kind: NodeKind::Drone {
                    pdr: opt.config.drone[0].pdr,
                    crashed: false,
                },
                window: Window::Main,
            },
            running: true,
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
        while self.running {
            terminal.draw(|frame| {
                crate::view::render(
                    &self.network,
                    &self.screen,
                    &mut self.node_list_state,
                    &mut self.packet_table_state,
                    frame,
                )
            })?;

            if let Some(message) = keypress_handler::handle_crossterm_events(&self.screen)? {
                self.transition(message);
            };

            while let Ok(event) = self.command_recv.try_recv() {
                if let DroneEvent::ControllerShortcut(packet) = event {
                    todo!()
                }
                self.save_event(event);
            }
        }

        Ok(())
    }

    fn save_event(&mut self, event: DroneEvent) {
        let packet = match event {
            DroneEvent::PacketSent(ref packet) => packet,
            DroneEvent::PacketDropped(ref packet) => packet,
            DroneEvent::ControllerShortcut(ref packet) => packet,
        };
        let id = packet.routing_header.hops[packet.routing_header.hop_index - 1];
        debug!("Drone {id} sent event PacketSent with packet {packet}");

        if let Some(node) = self.network.get_mut_node_from_id(id) {
            match event {
                DroneEvent::PacketSent(ref packet) => {
                    node.sent.push_front(packet.clone());
                }
                DroneEvent::PacketDropped(ref packet) => node.dropped.push_front(packet.clone()),
                DroneEvent::ControllerShortcut(ref packet) => {
                    node.shortcutted.push_front(packet.clone())
                }
            }

            if id == self.screen.focus {
                if let Window::Detail { tab } = self.screen.window {
                    let isdrone = matches!(self.screen.kind, NodeKind::Drone { .. });
                    match event {
                        DroneEvent::PacketSent(_) if tab == 0 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        DroneEvent::PacketDropped(_) if tab == 1 && isdrone => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        DroneEvent::ControllerShortcut(_) if tab == 2 && isdrone => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn add_connection(&mut self, from: NodeId, to: NodeId) {
        //check connection is not between two clients/servers
        if let (Some(nfrom), Some(nto)) = (
            self.network.get_node_from_id(from),
            self.network.get_node_from_id(to),
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
            let _ = command_sender_from.send(DroneCommand::AddSender(to, packet_sender_to.clone()));
            command_sender_to.send(DroneCommand::AddSender(from, packet_sender_from.clone()));

            // for now we assume they succesfully added channel, and show it in the model
            self.network.add_edge(from, to);
            // TODO: select correct node
            //self.model.select_node(from);
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

        // TODO:  tell the other drones to remove edges that point to crashed drone

        // set in the model the corresponding node to crashed true
        self.network.crash_drone(id);
    }

    /// TODO: adds to the model and to the simulation the given node
    fn add_node(&mut self, node: NodeRepresentation) {
        // TODO: add node here instead
        if let Some(n) = self.network.get_node_from_id(node.id) {
            match n.kind {
                NodeKind::Drone { pdr, crashed } => todo!(),
                NodeKind::Client => todo!(),
                NodeKind::Server => todo!(),
            }
        } else {
            //todo:improve
            panic!("added drone not found");
        }
        //self.node_list_state.select_last();
    }
    fn change_pdr(&mut self, pdr: f64) {
        todo!();
    }
    fn scroll_list(&mut self, up: bool) {
        if up {
            self.node_list_state.scroll_up_by(1);
        } else {
            self.node_list_state.scroll_down_by(1);
        }

        if let Some(selected) = self.node_list_state.selected() {
            if let Some(node) = self.network.get_node_from_pos(selected) {
                self.screen.focus = node.id;
                self.screen.kind = node.kind;
            }
        }
    }
}

impl MySimulationController {
    fn transition(&mut self, message: AppMessage) {
        let kind = self.screen.kind;
        let id = self.screen.focus;
        match message {
            AppMessage::Quit => self.running = false,
            AppMessage::Crash => match self.screen.window {
                Window::Detail { tab: _ } if matches!(kind, NodeKind::Drone { .. }) => {
                    // TODO: first check that actual crash was succesful?
                    self.crash(id);
                    self.network.crash_drone(id);
                    self.screen.window = Window::Main;
                }
                _ => {}
            },
            // for Detail
            AppMessage::ChangeTab => {
                if let Window::Detail { ref mut tab } = self.screen.window {
                    *tab = tab.saturating_add(1);
                    self.packet_table_state.select(Some(0));
                    if let NodeKind::Drone { .. } = kind {
                        *tab %= 3;
                    } else {
                        *tab %= 3;
                    }
                }
            }
            // for add node
            AppMessage::SetNodeKind(kind) => {
                if let Window::AddNode { ref mut toadd } = self.screen.window {
                    toadd.kind = kind;
                }
            }
            // Window changes
            AppMessage::WindowAddConnection => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::AddConnection { origin: id }
                }
            }
            AppMessage::WindowAddNode => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::AddNode {
                        toadd: NodeRepresentation::default(),
                    }
                }
            }
            AppMessage::WindowChangePDR => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::ChangePdr { pdr: 0.05 }
                }
            }
            AppMessage::WindowMove => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::Move
                }
            }
            AppMessage::WindowDetail => {
                if let Window::Main = self.screen.window {
                    // TODO: decide if you need to check if there is one item in the needed
                    // vecdeque
                    self.packet_table_state.select(Some(0));
                    self.screen.window = Window::Detail { tab: 0 }
                }
            }
            AppMessage::Done => match self.screen.window {
                Window::Main => {}
                Window::Move | Window::Detail { tab: _ } => self.screen.window = Window::Main,
                Window::AddNode { ref toadd } => {
                    self.add_node(toadd.clone());
                    self.screen.window = Window::Main
                }
                Window::AddConnection { origin } => {
                    self.add_connection(origin, id);
                    self.screen.window = Window::Main
                }
                Window::ChangePdr { pdr } => {
                    self.change_pdr(pdr);
                    self.screen.window = Window::Main;
                }
            },
            // List movement
            AppMessage::ScrollUp => match self.screen.window {
                // TODO: check not adding connection client client, also in scrolldown
                Window::Main | Window::AddConnection { .. } => {
                    self.scroll_list(true);
                }
                Window::Detail { tab } => {
                    self.packet_table_state.scroll_up_by(1);
                }
                _ => {}
            },
            AppMessage::ScrollDown => match self.screen.window {
                Window::Main | Window::AddConnection { .. } => {
                    self.scroll_list(false);
                }
                Window::Detail { tab } => {
                    self.packet_table_state.scroll_down_by(1);
                }
                _ => {}
            },
            // Node movement
            AppMessage::MoveNode { x, y } => {
                let node = self.network.get_mut_node_from_id(id).unwrap();
                if x > 0 {
                    node.shiftr(x as u32);
                } else {
                    node.shiftl(x.unsigned_abs() as u32);
                }
                if y > 0 {
                    node.shiftu(y as u32);
                } else {
                    node.shiftd(y.unsigned_abs() as u32);
                }
            }
        }
    }
}
