use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{self, unbounded, Receiver, Sender};
use log::LevelFilter;
use messages::{
    node_event::NodeEvent, ChatRequest, ChatResponse, ErrorType, MediaRequest, MediaResponse,
    MessageType, RequestType, ResponseType, ServerType, TextRequest, TextResponse,
};
use ratatui::{backend::TestBackend, Terminal};
use simplelog::{format_description, ConfigBuilder, ThreadLogMode, ThreadPadding, WriteLogger};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{
        Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType,
    },
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
    HashMap<NodeId, Receiver<Packet>>,
) {
    start_dummy_sc_from_cfg_with_handles(config, HashMap::new())
}

#[cfg(feature = "integration_tests")]
pub fn start_dummy_sc_from_cfg_with_handles(
    config: &str,
    mut node_handles: HashMap<u8, JoinHandle<()>>,
) -> (
    Sender<KeyEvent>,
    JoinHandle<()>,
    Sender<DroneEvent>,
    Sender<NodeEvent>,
    HashMap<NodeId, Receiver<DroneCommand>>,
    HashMap<NodeId, Receiver<Packet>>,
) {
    let add_handles = node_handles.is_empty();
    use std::time::Duration;

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
        if add_handles {
            let handle = thread::Builder::new()
                .name(format!("drone#{}", n.id))
                .spawn(move || loop {
                    thread::sleep(Duration::from_secs(1));
                })
                .expect("could not create thread");
            node_handles.insert(n.id, handle);
        }
    }

    for n in config.server.iter() {
        let (cs, cr) = unbounded::<DroneCommand>();
        command_receivers.insert(n.id, cr);
        command_senders.insert(n.id, cs);
        let (ps, pr) = unbounded::<Packet>();
        packet_receivers.insert(n.id, pr);
        packet_senders.insert(n.id, ps);
        if add_handles {
            let handle = thread::Builder::new()
                .name(format!("server#{}", n.id))
                .spawn(move || loop {
                    thread::sleep(Duration::from_secs(1));
                })
                .expect("could not create thread");
            node_handles.insert(n.id, handle);
        }
    }

    for n in config.client.iter() {
        let (cs, cr) = unbounded::<DroneCommand>();
        command_receivers.insert(n.id, cr);
        command_senders.insert(n.id, cs);
        let (ps, pr) = unbounded::<Packet>();
        packet_receivers.insert(n.id, pr);
        packet_senders.insert(n.id, ps);
        if add_handles {
            let handle = thread::Builder::new()
                .name(format!("client#{}", n.id))
                .spawn(move || loop {
                    thread::sleep(Duration::from_secs(1));
                })
                .expect("could not create thread");
            node_handles.insert(n.id, handle);
        }
    }

    let opt = SimControllerOptions {
        command_send: command_senders,
        droneevent_recv,
        droneevent_send: droneevent_send.clone(),
        nodeevent_recv,
        nodeevent_send: nodeevent_send.clone(),
        packet_send: packet_senders,
        config,
        node_handles,
    };

    let terminal = Terminal::new(TestBackend::new(50, 50)).unwrap();
    let mut simcontr = MySimulationController::new(opt);
    simcontr.set_keyevent_recv(keyevent_recv);
    let join_handle = thread::spawn(move || {
        //simcontr.run();
        simcontr.run_with_terminal(terminal);
    });
    (
        appmess_send,
        join_handle,
        droneevent_send,
        nodeevent_send,
        command_receivers,
        packet_receivers,
    )
}

pub fn random_packet() -> PacketType {
    match rand::random_range(1..=5u64) {
        1 => PacketType::Nack(wg_2024::packet::Nack {
            fragment_index: 34,
            nack_type: NackType::Dropped,
        }),
        2 => PacketType::Ack(Ack {
            fragment_index: rand::random_range(1..3456),
        }),
        3 => PacketType::MsgFragment(Fragment {
            fragment_index: rand::random_range(1..256),
            total_n_fragments: rand::random_range(1..345),
            length: rand::random_range(1..128),
            data: [0; 128],
        }),
        4 => PacketType::FloodRequest(FloodRequest {
            flood_id: 3,
            initiator_id: 9,
            path_trace: vec![(rand::random_range(1..=10), NodeType::Drone)],
        }),
        _ => PacketType::FloodResponse(FloodResponse {
            flood_id: 2,
            path_trace: vec![],
        }),
    }
}

pub fn random_mtype() -> MessageType {
    match rand::random_range(1..=9u64) {
        1 => messages::MessageType::Request(messages::RequestType::DiscoveryRequest(())),
        2 => messages::MessageType::Request(messages::RequestType::MediaRequest(
            MediaRequest::MediaList,
        )),
        3 => messages::MessageType::Request(messages::RequestType::ChatRequest(
            messages::ChatRequest::ClientList,
        )),
        4 => messages::MessageType::Request(messages::RequestType::TextRequest(
            messages::TextRequest::TextList,
        )),
        5 => messages::MessageType::Response(messages::ResponseType::TextResponse(
            messages::TextResponse::NotFound,
        )),
        7 => messages::MessageType::Response(messages::ResponseType::ChatResponse(
            messages::ChatResponse::MessageSent,
        )),
        8 => messages::MessageType::Response(messages::ResponseType::MediaResponse(
            messages::MediaResponse::MediaList(vec![]),
        )),
        _ => messages::MessageType::Response(messages::ResponseType::DiscoveryResponse(
            ServerType::ContentServer,
        )),
    }
}

pub fn expect_command(rcv: &Receiver<DroneCommand>, command: &DroneCommand) {
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

pub fn expect_no_command(rcv: &Receiver<DroneCommand>) {
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
            expect_command(r, command);
        } else {
            expect_no_command(r);
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
            expect_command(r, command);
        }
    }
}

pub fn expect_packet(rcv: &Receiver<Packet>, packet: &Packet) {
    match rcv.try_recv() {
        Ok(c) => {
            if c != *packet {
                panic!("received packet: {:?} was expecting {:?}", c, packet)
            }
        }
        Err(e) => {
            panic!("got {e}, was expecting {:?}", packet)
        }
    }
}

pub fn expect_no_packet(rcv: &Receiver<Packet>) {
    match rcv.try_recv() {
        Ok(c) => {
            panic!("received packet: {:?} was expecting nothing", c)
        }
        Err(e) => {}
    }
}

pub fn expect_just_packet_hmap(rcv: &HashMap<u8, Receiver<Packet>>, id: u8, packet: &Packet) {
    for (n, r) in rcv {
        if *n == id {
            expect_packet(r, packet);
        } else {
            expect_no_packet(r);
        }
    }
}

pub fn expect_packet_hmap(rcv: &HashMap<u8, Receiver<Packet>>, id: u8, packet: &Packet) {
    for (n, r) in rcv {
        if *n == id {
            expect_packet(r, packet);
        }
    }
}

pub fn all_the_message_types() -> Vec<MessageType> {
    vec![
        MessageType::Request(RequestType::TextRequest(TextRequest::TextList)),
        MessageType::Request(RequestType::TextRequest(TextRequest::Text(42))),
        MessageType::Request(RequestType::MediaRequest(MediaRequest::MediaList)),
        MessageType::Request(RequestType::MediaRequest(MediaRequest::Media(42))),
        MessageType::Request(RequestType::ChatRequest(ChatRequest::ClientList)),
        MessageType::Request(RequestType::ChatRequest(ChatRequest::Register(
            NodeId::default(),
        ))),
        MessageType::Request(RequestType::ChatRequest(ChatRequest::SendMessage {
            from: NodeId::default(),
            to: NodeId::default(),
            message: "Hello".to_string(),
        })),
        MessageType::Request(RequestType::DiscoveryRequest(())),
        MessageType::Response(ResponseType::TextResponse(TextResponse::TextList(vec![
            1, 2, 3,
        ]))),
        MessageType::Response(ResponseType::TextResponse(TextResponse::Text(
            "Hello".to_string(),
        ))),
        MessageType::Response(ResponseType::TextResponse(TextResponse::NotFound)),
        MessageType::Response(ResponseType::MediaResponse(MediaResponse::MediaList(vec![
            1, 2, 3,
        ]))),
        MessageType::Response(ResponseType::MediaResponse(MediaResponse::Media(vec![
            1, 2, 3,
        ]))),
        MessageType::Response(ResponseType::ChatResponse(ChatResponse::ClientList(vec![
            NodeId::default(),
        ]))),
        MessageType::Response(ResponseType::ChatResponse(ChatResponse::MessageFrom {
            from: NodeId::default(),
            message: vec![72, 101, 108, 108, 111],
        })),
        MessageType::Response(ResponseType::ChatResponse(ChatResponse::MessageSent)),
        MessageType::Response(ResponseType::DiscoveryResponse(
            ServerType::CommunicationServer,
        )),
        MessageType::Response(ResponseType::DiscoveryResponse(ServerType::ContentServer)),
        MessageType::Error(ErrorType::Unsupported(RequestType::TextRequest(
            TextRequest::TextList,
        ))),
        MessageType::Error(ErrorType::Unexpected(ResponseType::TextResponse(
            TextResponse::NotFound,
        ))),
    ]
}

pub fn all_the_packet_types(from: u8) -> Vec<PacketType> {
    let fragment_index = 0;
    let src = from;
    let total_n_fragments = 1;

    let flood_id = 0;
    let initiator_id = 0;
    let path_trace = vec![(from - 1, NodeType::Drone), (from, NodeType::Drone)];
    vec![
        PacketType::Ack(Ack { fragment_index }),
        PacketType::Nack(Nack {
            fragment_index,
            nack_type: NackType::Dropped,
        }),
        PacketType::Nack(Nack {
            fragment_index,
            nack_type: NackType::DestinationIsDrone,
        }),
        PacketType::Nack(Nack {
            fragment_index,
            nack_type: NackType::ErrorInRouting(src),
        }),
        PacketType::Nack(Nack {
            fragment_index,
            nack_type: NackType::UnexpectedRecipient(src),
        }),
        PacketType::MsgFragment(Fragment {
            fragment_index,
            total_n_fragments,
            length: 128,
            data: [0; 128],
        }),
        PacketType::FloodRequest(FloodRequest {
            flood_id,
            initiator_id,
            path_trace: path_trace.clone(),
        }),
        PacketType::FloodResponse(FloodResponse {
            flood_id,
            path_trace,
        }),
    ]
}
