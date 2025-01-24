mod keypress_handler;
mod network;
mod screen;
mod utilities;
mod view;
use messages::node_event::NodeEvent;
use messages::Message;

use crate::screen::Screen;
use core::f32;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    thread::{Builder, JoinHandle},
};

use crate::network::Network;
use crossbeam_channel::{select, select_biased, unbounded, Receiver, Sender};
use log::{debug, error, info, trace, warn};
use network::{node_kind::NodeKind, node_representation::NodeRepresentation};
use rand::random;
use ratatui::{
    widgets::{ListState, TableState},
    DefaultTerminal,
};
use screen::Window;
use utilities::app_message::AppMessage;
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

pub struct SimControllerOptions {
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub command_send: HashMap<NodeId, Sender<DroneCommand>>,
    pub droneevent_send: Sender<DroneEvent>,
    pub droneevent_recv: Receiver<DroneEvent>,
    pub nodeevent_send: Sender<NodeEvent>,
    pub nodeevent_recv: Receiver<NodeEvent>,
    pub node_handles: HashMap<NodeId, JoinHandle<()>>,
    pub config: Config,
}

pub struct MySimulationController {
    // external comms
    packet_send: HashMap<NodeId, Sender<Packet>>,
    command_send: HashMap<NodeId, Sender<DroneCommand>>,
    nodeevent_send: Sender<NodeEvent>,
    nodeevent_recv: Receiver<NodeEvent>,
    droneevent_send: Sender<DroneEvent>,
    droneevent_recv: Receiver<DroneEvent>,
    node_handles: HashMap<NodeId, JoinHandle<()>>,
    // internal state
    running: bool,
    network: Network,
    node_list_state: ListState,
    packet_table_state: TableState,
    screen: Screen,
}

impl MySimulationController {
    pub fn new(opt: SimControllerOptions) -> Self {
        info!("created SC");
        MySimulationController {
            command_send: opt.command_send,
            droneevent_recv: opt.droneevent_recv,
            droneevent_send: opt.droneevent_send,
            nodeevent_send: opt.nodeevent_send,
            nodeevent_recv: opt.nodeevent_recv,
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
    /// runs the main loop of the sc, repeats the following:
    /// - draw tui
    /// - handle keypresses
    /// - handle all DroneEvents
    fn start(&mut self, mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
        info!("started SC");
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
                debug!("received AppMessage: {:?}", message);
                self.transition(message);
            };
            loop {
                select! {
                    recv(self.droneevent_recv)->res =>{
                        if let Ok(event) = res{
                            if let DroneEvent::ControllerShortcut(packet) = event {
                                todo!()
                            }
                            self.save_droneevent(event);
                        }
                    },
                    recv(self.nodeevent_recv)-> res =>{
                        if let Ok(event) = res{
                            self.save_nodeevent(event);
                        }

                    }

                    default => break,
                }
            }
        }

        Ok(())
    }

    fn save_nodeevent(&mut self, event: NodeEvent) {
        let id = event
            .source()
            .expect("routing header does not have previous hop");

        if let Some(node) = self.network.get_mut_node_from_id(id) {
            // fix scrolling pushdown on certain tabs
            if let Window::Detail { tab } = self.screen.window {
                if id == self.screen.focus {
                    match event {
                        NodeEvent::PacketSent(_) if tab == 0 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        NodeEvent::StartingMessageTransmission(_) if tab == 1 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        NodeEvent::MessageReceived(_) if tab == 2 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        _ => {}
                    }
                }
            }
            // save the data received in the correct location
            match event {
                NodeEvent::PacketSent(packet) => {
                    trace!("Client/Server #{id} sent event PacketSent with packet {packet}");
                    node.sent.push_front(packet.clone());
                }
                NodeEvent::MessageSentSuccessfully(message) => {
                    debug!(
                        "Client/Server #{id} sent event MessageSentSuccessfully with Message {:?}",
                        message
                    );
                    node.msent.insert(message.session_id, (message, true));
                }
                NodeEvent::StartingMessageTransmission(message) => {
                    debug!(
                        "Client/Server #{id} sent event StartingMessageTransmission with Message {:?}",
                        message
                    );
                    node.msent.insert(message.session_id, (message, false));
                }
                NodeEvent::MessageReceived(message) => {
                    debug!(
                        "Client/Server #{id} sent event MessageReceived with Message {:?}",
                        message
                    );
                    node.mreceived.push_front(message);
                }
                NodeEvent::KnownNetworkGraph { source: _, graph } => {
                    debug!(
                        "Client/Server #{id} sent event KnownNetworkGraph with Network {:?}",
                        graph
                    );
                    node.knowntopology = graph;
                }
            };
        }
    }

    /// saves inside the NodeRepresentation the events received on the sc channel, logs the event
    /// received, and in the case of the Detail window, it scrolls the table state to match the
    /// newly added packet
    fn save_droneevent(&mut self, event: DroneEvent) {
        let packet = match event {
            DroneEvent::PacketSent(ref packet) => packet,
            DroneEvent::PacketDropped(ref packet) => packet,
            DroneEvent::ControllerShortcut(ref packet) => packet,
        };
        let id = packet.routing_header.hops[packet.routing_header.hop_index - 1];

        if let Some(node) = self.network.get_mut_node_from_id(id) {
            match event {
                DroneEvent::PacketSent(ref packet) => {
                    trace!("Drone {id} sent event PacketSent with packet {packet}");
                    node.sent.push_front(packet.clone());
                }
                DroneEvent::PacketDropped(ref packet) => {
                    trace!("Drone {id} sent event PacketDropped with packet {packet}");
                    node.dropped.push_front(packet.clone())
                }
                DroneEvent::ControllerShortcut(ref packet) => {
                    debug!("Drone {id} sent event ControllerShortcut with packet {packet}");
                    node.shortcutted.push_front(packet.clone())
                }
            }

            if id == self.screen.focus {
                if let Window::Detail { tab } = self.screen.window {
                    match event {
                        DroneEvent::PacketSent(_) if tab == 0 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        DroneEvent::PacketDropped(_) if tab == 1 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        DroneEvent::ControllerShortcut(_) if tab == 2 => {
                            self.packet_table_state.scroll_down_by(1)
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// adds a connection between two nodes, first checking that the given source and destination
    /// follow certain rules(connections does not exist, at least one is a drone, not between same
    /// node,none of them crashed), then sends to the corresponding nodes in the simulation the command to add a
    /// neighbor, if unsuccessful returns an error with explanation, when the error is unexpected
    /// it panics
    fn add_connection(&self, from: NodeId, to: NodeId) -> Result<(), &'static str> {
        debug!("checking if connection to be added is not between two client/server, not between same node, does not exist...");
        if let (Some(nfrom), Some(nto)) = (
            self.network.get_node_from_id(from),
            self.network.get_node_from_id(to),
        ) {
            match (nfrom.kind, nto.kind) {
                (
                    NodeKind::Drone {
                        pdr: _,
                        crashed: true,
                    },
                    _,
                )
                | (
                    _,
                    NodeKind::Drone {
                        pdr: _,
                        crashed: true,
                    },
                ) => {
                    warn!(
                        "Cannot connect {} and {}, one or both of them are a crashed drone",
                        nfrom.kind, nto.kind
                    );
                    return Err("Cannot connect to a crashed drone");
                }
                (NodeKind::Client | NodeKind::Server, NodeKind::Client | NodeKind::Server) => {
                    warn!(
                        "Cannot connect {} and {}, at least one should be a drone",
                        nfrom.kind, nto.kind
                    );
                    return Err("trying to connect two Clients/Servers");
                }
                _ => {}
            }
            if nfrom.id == nto.id {
                warn!(
                    "Cannot connect {} and {}, they are the same node ",
                    nfrom.id, nto.id
                );
                return Err("trying to connect a node with itself");
            }
            if nfrom.adj.contains(&to) || nto.adj.contains(&from) {
                warn!(
                    "Cannot connect {} and {}, they are already connected",
                    nfrom.id, nto.id
                );
                return Err("trying to connect two already connected nodes");
            }
        } else {
            panic!("nodes to connect not found: {from} and {to} are not present in the network representation");
        }

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
            self.packet_send.get(&from),
            self.packet_send.get(&to),
        ) {
            let _ = command_sender_from.send(DroneCommand::AddSender(to, packet_sender_to.clone()));
            let _ =
                command_sender_to.send(DroneCommand::AddSender(from, packet_sender_from.clone()));

            Ok(())
        } else {
            error!(
                "could not find command senders or packet senders for nodes with id {} and {}",
                from, to
            );
            error!("packet senders: {:?}", self.packet_send);
            error!("command senders: {:?}", self.command_send);
            panic!("could not create connection")
        }
    }

    /// send crash command to drone, removesender command to neighbors
    /// # Panics
    /// - if there is no noderepresentation in the network for the crashing drone
    /// - if the id is not of a drone
    /// - if there is no command sender for the drone or any of its neighbors
    /// - if there is no packet sender for the drone
    fn crash(&mut self, id: NodeId) -> Result<(), &'static str> {
        if let Some(drone_command_sender) = self.command_send.get(&id) {
            // send command to corresponding drone to crash
            let _ = drone_command_sender.send(DroneCommand::Crash);
            let node = self
                .network
                .get_node_from_id(id)
                .expect("could not find noderepresentation for drone {id}");
            if !matches!(node.kind, NodeKind::Drone { .. }) {
                panic!("trying to crash non-drone node #{id}")
            }
            let nodes = node.adj.clone();
            for n in nodes {
                let sender = self
                    .command_send
                    .get(&n)
                    .expect("could not find comm sender for drone {n}");

                // send command to neighbor drones to remove sender
                let _ = sender.send(DroneCommand::RemoveSender(id));
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

    /// generates a random id for a node, different from any of the other nodes in the network
    fn random_unique_id(&self) -> NodeId {
        loop {
            let id = random::<u8>();
            if self.network.get_node_from_id(id).is_none() {
                return id;
            }
        }
    }

    /// creates drone with random id, spawns its thread, and adds it to the SC
    /// # Panics
    /// panics if it can't create the drone thread
    fn spawn_drone(&mut self) {
        let kind = NodeKind::Drone {
            pdr: 0.05,
            crashed: false,
        };
        let n = NodeRepresentation::new(self.random_unique_id(), 0, 0, kind, HashSet::new());

        let event_send = self.droneevent_send.clone();
        let (command_send, command_recv) = unbounded::<DroneCommand>();
        let (packet_send, packet_recv) = unbounded::<Packet>();

        self.command_send.insert(n.id, command_send);
        self.packet_send.insert(n.id, packet_send);

        let handle = Builder::new()
            .name(format!("NullPointer#{}", n.id))
            .spawn(move || {
                null_pointer_drone::MyDrone::new(
                    n.id,
                    event_send,
                    command_recv,
                    packet_recv,
                    HashMap::new(),
                    0.05,
                )
                .run()
            })
            .expect("could not spawn drone thread");

        self.node_handles.insert(n.id, handle);

        self.network.nodes.push(n.clone());

        self.node_list_state.select_last();
        self.screen.focus = n.id;
        self.screen.kind = kind;
    }

    /// changes pdr of drone in representation and of actual drone
    /// Panics
    /// panics if it can't find drone in the network, if it's not a drone, if it's crashed, if
    /// there is no command sender for it
    fn change_pdr(&mut self, newpdr: f32) {
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
            _ => panic!("either not drone or crashed"),
        }
    }

    /// scrolls list either up or down, then  updates focus and kind accordingly
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

    /// resets list to first node, then updates focus and kind accordingly
    fn reset_list(&mut self) {
        self.node_list_state.select_first();

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
            AppMessage::Quit => {
                info!("received AppMessage::Quit, exiting...");
                self.running = false
            }
            AppMessage::Crash => match self.screen.window {
                Window::Detail { tab: _ } if matches!(kind, NodeKind::Drone { .. }) => {
                    match self.crash(id) {
                        Ok(_) => {
                            // mark the drone as crashed in the network
                            self.network.crash_drone(id);
                            self.screen.window = Window::Main;
                            // TODO at some point check that the drone's thread actually returns
                        }
                        Err(message) => {
                            debug!("error crashing drone, switching to Window::Error");
                            self.screen.window = Window::Error { message };
                        }
                    };
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
            // spawn drone
            AppMessage::SpawnDrone => {
                if let Window::Main = self.screen.window {
                    self.spawn_drone()
                }
            }
            // Window changes
            AppMessage::WindowAddConnection => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::AddConnection { origin: id }
                }
            }
            AppMessage::WindowChangePDR => {
                if let Window::Detail { tab: _ } = self.screen.window {
                    if let NodeKind::Drone {
                        pdr,
                        crashed: false,
                    } = kind
                    {
                        self.screen.window = Window::ChangePdr { pdr }
                    }
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
                Window::Error { message: _ } => {
                    self.reset_list();
                    self.screen.window = Window::Main;
                }
                Window::Move | Window::Detail { tab: _ } => self.screen.window = Window::Main,
                Window::AddConnection { origin } => {
                    info!("received AppMessage::Done, current window is AddConnection, adding connection...");
                    let res = self.add_connection(origin, id);
                    match res {
                        Ok(_) => {
                            self.network.add_edge(origin, id);
                            self.reset_list();
                            self.screen.window = Window::Main;
                            info!("connection added succesfully, switched back to Window::Main");
                        }
                        Err(s) => {
                            debug!("could not add connection, switching to Window::Error");
                            self.screen.window = Window::Error { message: s };
                        }
                    };
                }
                Window::ChangePdr { pdr } => {
                    self.change_pdr(pdr);
                    self.screen.window = Window::Detail { tab: 0 };
                }
            },
            // List movement
            AppMessage::ScrollUp => match self.screen.window {
                Window::Main | Window::AddConnection { .. } => {
                    self.scroll_list(true);
                }
                Window::Detail { tab } => {
                    self.packet_table_state.scroll_up_by(1);
                }
                Window::ChangePdr { ref mut pdr } => {
                    *pdr += 0.01;
                    if *pdr > 1.0 {
                        *pdr = 1.0
                    }
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
                Window::ChangePdr { ref mut pdr } => {
                    *pdr -= 0.01;
                    if *pdr < 0.0 {
                        *pdr = 0.0
                    }
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
