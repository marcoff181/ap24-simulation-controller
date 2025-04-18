#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

// graphics, keypress handling and helpers
mod keypress_handler;
mod network;
mod screen;
mod utilities;
mod view;

// impls for MySimulationController
mod event_saver;
mod interacts_with_simulation;
mod transition;

#[cfg(feature = "custom_terminal_backend")]
use ratatui::backend::TestBackend;

use crate::network::Network;
use crate::screen::Screen;
use crossbeam_channel::{select, Receiver, Sender};
#[cfg(feature = "appmessage_through_crossbeam")]
use crossterm::event::KeyEvent;
use log::{debug, info};
use messages::node_event::NodeEvent;
use network::node_kind::NodeKind;
use ratatui::{
    widgets::{ListState, TableState},
    Terminal,
};
use screen::Window;
use std::{collections::HashMap, thread::JoinHandle};

use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::Packet,
};

pub struct SimControllerOptions {
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub command_send: HashMap<NodeId, Sender<DroneCommand>>,
    pub droneevent_send: Sender<DroneEvent>,
    pub droneevent_recv: Receiver<DroneEvent>,
    pub nodeevent_recv: Receiver<NodeEvent>,
    pub node_handles: HashMap<NodeId, JoinHandle<()>>,
    pub config: Config,
    pub max_eventbuffer_dim: usize,
}

pub struct MySimulationController {
    #[cfg(feature = "appmessage_through_crossbeam")]
    keyevent_recv: Option<Receiver<KeyEvent>>,
    // external comms
    packet_send: HashMap<NodeId, Sender<Packet>>,
    command_send: HashMap<NodeId, Sender<DroneCommand>>,
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
    max_eventbuffer_dim: usize,
}

#[cfg(not(feature = "appmessage_through_crossbeam"))]
impl Drop for MySimulationController {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

impl MySimulationController {
    /// initializes the SC using the given options, by initializing the network and checking that
    /// it is valid
    /// # Panics
    /// Panics if the given configuration is invalid
    #[must_use]
    pub fn new(opt: SimControllerOptions) -> Self {
        info!("creating SC...");
        let mut network = match Network::new(&opt.config) {
            Ok(n) => n,
            Err(s) => panic!("when converting cfg to network found error: {s}"),
        };
        for (id, handle) in &opt.node_handles {
            if let Some(nrepr) = network.get_mut_node_from_id(*id) {
                if let Some(t) = handle.thread().name() {
                    nrepr.thread_name = t.to_string();
                };
            }
        }

        MySimulationController {
            #[cfg(feature = "appmessage_through_crossbeam")]
            keyevent_recv: None,
            command_send: opt.command_send,
            droneevent_recv: opt.droneevent_recv,
            droneevent_send: opt.droneevent_send,
            nodeevent_recv: opt.nodeevent_recv,
            packet_send: opt.packet_send,
            node_handles: opt.node_handles,
            network,
            node_list_state: ListState::default().with_selected(Some(0)),
            packet_table_state: TableState::default().with_selected(0),
            screen: Screen {
                // there must be at least a drone, and it is guaranteed that a drone will be the
                // first of the list
                focus: opt.config.drone[0].id,
                kind: NodeKind::Drone {
                    pdr: opt.config.drone[0].pdr,
                    crashed: false,
                },
                window: Window::Main,
            },
            running: true,
            max_eventbuffer_dim: opt.max_eventbuffer_dim,
        }
    }

    /// initializes ratatui, then starts internal SC loop, that will exit only when key 'q' is
    /// pressed or when a panic occurs
    /// # Panics
    /// there are two kinds of panics in the SC:
    /// - unreachable panics are meant to never be reached during runtime, and if they are, they
    ///     are most likely sign of a bug in the internal workings of the SC, like not finding packet
    ///     senders or command senders for a given drone
    /// - normal panics are instead used when the sc receives a NodeEvent/DroneEvent that can be
    ///     considered 'broken', for example hop index out of bounds, a empty hops vector
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn run(&mut self) {
        let terminal = ratatui::init();
        self.start(terminal);
        info!("sc exiting normally...");
        ratatui::restore();
    }

    #[cfg(feature = "custom_terminal_backend")]
    pub fn run_with_terminal(&mut self, terminal: Terminal<TestBackend>) {
        let _ = self.start(terminal);
    }

    #[cfg(feature = "appmessage_through_crossbeam")]
    pub fn set_keyevent_recv(&mut self, rcv: Receiver<KeyEvent>) {
        self.keyevent_recv = Some(rcv);
    }
}

impl MySimulationController {
    /// runs the main loop of the sc
    /// Panics
    /// - if a node thread exits and the node was not a crashing drone
    /// - when a channel receiver returns an error
    fn start<B: ratatui::backend::Backend>(&mut self, mut terminal: Terminal<B>) {
        info!("started SC");
        let mut finished: Vec<NodeId> = Vec::new();
        while self.running {
            // ---------------------------------------------------------------------------
            // draw interface
            // ---------------------------------------------------------------------------
            terminal
                .draw(|frame| {
                    crate::view::render(
                        &self.network,
                        &self.screen,
                        &mut self.node_list_state,
                        &mut self.packet_table_state,
                        frame,
                    );
                })
                .expect("terminal.draw exited with error");

            // ---------------------------------------------------------------------------
            // listen for keypresses
            // ---------------------------------------------------------------------------
            #[cfg(feature = "appmessage_through_crossbeam")]
            if let Some(rcv) = &self.keyevent_recv {
                if let Some(message) =
                    keypress_handler::handle_keypress_from_recv(&self.screen, rcv)
                {
                    debug!(
                        "received AppMessage through crossbeam channel: {:?}",
                        message
                    );
                    self.transition(&message);
                };
            };

            #[cfg(not(feature = "appmessage_through_crossbeam"))]
            #[cfg_attr(coverage_nightly, coverage(off))]
            if let Some(message) = keypress_handler::handle_crossterm_events(&self.screen) {
                debug!("received AppMessage: {:?}", message);
                self.transition(&message);
            };

            // ---------------------------------------------------------------------------
            // check if node threads exit
            // ---------------------------------------------------------------------------
            for (id, h) in &mut self.node_handles {
                if h.is_finished() {
                    finished.push(*id);
                }
            }
            while let Some(id) = finished.pop() {
                let h = self.node_handles.remove(&id).unwrap();
                let node = self
                    .network
                    .get_node_from_id(id)
                    .expect("could not find node for node_handle of id #{id}");
                let res = h.join();
                match (res, node.kind) {
                    (
                        Ok(()),
                        NodeKind::Drone {
                            pdr: _,
                            crashed: true,
                        },
                    ) => info!("Crashed drone #{id} exited successfully"),
                    (Ok(()), _) => {
                        panic!("Node #{id} unexpectedly exited thread successfully")
                    }
                    (Err(e), _) => {
                        panic!(
                            "Node #{id} unexpectedly exited thread with an error: {:?}",
                            e.downcast_ref::<&str>()
                                .copied()
                                .or_else(|| e.downcast_ref::<String>().map(|x| x.as_str()))
                        )
                    }
                }
            }

            // ---------------------------------------------------------------------------
            // go through all NodeEvents and DroneEvents
            // ---------------------------------------------------------------------------
            loop {
                select! {
                                    recv(self.droneevent_recv)->res =>{
                                        match res{
                    Ok(event) => {

                                            if let DroneEvent::ControllerShortcut(ref packet) = event {
                                                self.shortcut_packet(packet.clone());
                                            }
                                            self.save_droneevent(event);
                    },
                    Err(err) => {
                                           panic!("error for nodevent receiver: {err:?}");
                        },
                    }
                                        },
                                        recv(self.nodeevent_recv)-> res =>{
                                        match res{
                    Ok(event) => {

                                            self.save_nodeevent(event);
                    },
                    Err(err) => {
                                            panic!("error for nodevent receiver: {err:?}");
                    },
                }

                                    }
                                    default => break,
                                }
            }
        }
    }

    /// generates a random id for a node, different from any of the other nodes in the network
    fn random_unique_id(&self) -> NodeId {
        let mut id = 1u8;
        loop {
            if self.network.get_node_from_id(id).is_none() {
                return id;
            }
            id += 1;
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
