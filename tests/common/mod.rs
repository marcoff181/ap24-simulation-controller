use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{self, unbounded, Receiver, Sender};
use log::LevelFilter;
use messages::{node_event::NodeEvent, MediaRequest, MessageType, ServerType};
use ratatui::{backend::TestBackend, Terminal};
use simplelog::{format_description, ConfigBuilder, ThreadLogMode, ThreadPadding, WriteLogger};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, NackType, NodeType, Packet, PacketType},
};

#[cfg(feature = "integration_tests")]
use crossterm::event::KeyEvent;
#[cfg(feature = "integration_tests")]
pub fn start_dummy_sc_from_cfg(
    config: &str,
) -> (
    Sender<KeyEvent>,
    JoinHandle<()>,
    Sender<DroneEvent>,
    Sender<NodeEvent>,
    HashMap<NodeId, Receiver<DroneCommand>>,
) {
    use ap24_simulation_controller::{AppMessage, MySimulationController, SimControllerOptions};

    let config_data = std::fs::read_to_string(config).expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    let (droneevent_send, droneevent_recv) = unbounded::<DroneEvent>();
    let (nodeevent_send, nodeevent_recv) = unbounded::<NodeEvent>();
    let (appmess_send, keyevent_recv) = unbounded::<KeyEvent>();

    let mut command_receivers: HashMap<NodeId, Receiver<DroneCommand>> = HashMap::new();
    let mut command_senders: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();
    let mut packet_receivers: HashMap<NodeId, Receiver<Packet>> = HashMap::new();
    let mut packet_senders: HashMap<NodeId, Sender<Packet>> = HashMap::new();

    for n in config.drone.iter() {
        let (cs, cr) = unbounded::<DroneCommand>();
        command_receivers.insert(n.id, cr);
        command_senders.insert(n.id, cs);
        let (ps, pr) = unbounded::<Packet>();
        packet_receivers.insert(n.id, pr);
        packet_senders.insert(n.id, ps);
    }

    for n in config.server.iter() {
        let (cs, cr) = unbounded::<DroneCommand>();
        command_receivers.insert(n.id, cr);
        command_senders.insert(n.id, cs);
        let (ps, pr) = unbounded::<Packet>();
        packet_receivers.insert(n.id, pr);
        packet_senders.insert(n.id, ps);
    }

    for n in config.client.iter() {
        let (cs, cr) = unbounded::<DroneCommand>();
        command_receivers.insert(n.id, cr);
        command_senders.insert(n.id, cs);
        let (ps, pr) = unbounded::<Packet>();
        packet_receivers.insert(n.id, pr);
        packet_senders.insert(n.id, ps);
    }

    let opt = SimControllerOptions {
        command_send: command_senders,
        droneevent_recv,
        droneevent_send: droneevent_send.clone(),
        nodeevent_recv,
        nodeevent_send: nodeevent_send.clone(),
        packet_send: packet_senders,
        config,
        node_handles: HashMap::new(),
    };

    let terminal = Terminal::new(TestBackend::new(50, 50)).unwrap();
    let mut simcontr = MySimulationController::new(opt);
    simcontr.set_keyevent_recv(keyevent_recv);
    let join_handle = thread::spawn(move || {
        simcontr.run_with_terminal(terminal);
    });
    (
        appmess_send,
        join_handle,
        droneevent_send,
        nodeevent_send,
        command_receivers,
    )
}

pub fn expect_command_(rcv: &Receiver<DroneCommand>, command: &DroneCommand) {
    match rcv.try_recv() {
        Ok(c) => {
            if c != *command {
                panic!("received command: {:?} was expecting {:?}", c, command)
            }
        }
        Err(e) => {
            panic!("got {e}, was expecting {:?}", command)
        }
    }
}

pub fn expect_no_command_(rcv: &Receiver<DroneCommand>) {
    match rcv.try_recv() {
        Ok(c) => {
            panic!("received command: {:?} was expecting nothing", c)
        }
        Err(e) => {}
    }
}

pub fn expect_just_command_hmap(
    rcv: &HashMap<u8, Receiver<DroneCommand>>,
    id: u8,
    command: &DroneCommand,
) {
    for (n, r) in rcv {
        if *n == id {
            expect_command_(r, command);
        } else {
            expect_no_command_(r);
        }
    }
}

pub fn expect_command_hmap(
    rcv: &HashMap<u8, Receiver<DroneCommand>>,
    id: u8,
    command: &DroneCommand,
) {
    for (n, r) in rcv {
        if *n == id {
            expect_command_(r, command);
        }
    }
}
