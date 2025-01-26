use std::{
    collections::{HashMap, HashSet},
    task::Wake,
    thread,
    time::Duration,
};

use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
use crossbeam_channel::{self, unbounded, Receiver, Sender};
use log::LevelFilter;
use messages::{node_event::NodeEvent, MediaRequest, MessageType, ServerType};
use simplelog::{format_description, ConfigBuilder, ThreadLogMode, ThreadPadding, WriteLogger};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, NackType, NodeType, Packet, PacketType},
};
// used while developing to check how the GUI is functioning
fn main() {
    let log_level = LevelFilter::Debug;
    let _logger = WriteLogger::init(
        log_level,
        ConfigBuilder::new()
            //.set_thread_mode(ThreadLogMode::Both)
            .set_thread_level(LevelFilter::Error)
            //.set_thread_padding(ThreadPadding::Right(15))
            .build(),
        std::fs::File::create("app.log").expect("Could not create log file"),
    );

    let config_data = std::fs::read_to_string("./tests/config_files/input.toml")
        .expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    let (droneevent_send, droneevent_recv) = unbounded::<DroneEvent>();
    let (nodeevent_send, nodeevent_recv) = unbounded::<NodeEvent>();

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
    let mut simcontr = MySimulationController::new(opt);
    let join_handle = thread::spawn(move || {
        simcontr.run();
    });
    let mut sent_sid = HashSet::new();
    let mut sid = 0;
    loop {
        sid += 1;
        if join_handle.is_finished() {
            println!("Simulation Controller has finished its work.");
            break;
        }

        for id in 1..=6u8 {
            match id {
                1..=3 => {
                    let send = droneevent_send.send(DroneEvent::PacketSent(Packet {
                        pack_type: random_packet(),
                        routing_header: wg_2024::network::SourceRoutingHeader {
                            hop_index: 3,
                            hops: vec![2, 3, id, rand::random_range(1..=6)],
                        },
                        session_id: sid,
                    }));
                    let send = droneevent_send.send(DroneEvent::PacketDropped(Packet {
                        pack_type: PacketType::MsgFragment(Fragment {
                            fragment_index: rand::random_range(1..256),
                            total_n_fragments: rand::random_range(1..345),
                            length: rand::random_range(1..128),
                            data: [35; 128],
                        }),
                        routing_header: wg_2024::network::SourceRoutingHeader {
                            hop_index: 3,
                            hops: vec![2, 3, id, rand::random_range(1..=6)],
                        },
                        session_id: sid,
                    }));
                }
                _ => {
                    let send = nodeevent_send.send(NodeEvent::PacketSent(Packet {
                        pack_type: random_packet(),
                        routing_header: wg_2024::network::SourceRoutingHeader {
                            hop_index: 3,
                            hops: vec![2, 3, id, rand::random_range(1..=6)],
                        },
                        session_id: sid,
                    }));
                    if rand::random_bool(0.4) {
                        let send = nodeevent_send.send(NodeEvent::StartingMessageTransmission(
                            messages::Message {
                                source_id: id,
                                session_id: sid,
                                content: random_mtype(),
                            },
                        ));
                        sent_sid.insert((id, sid));
                    }
                    if rand::random_bool(0.3) {
                        if let Some(&element) = sent_sid.iter().next() {
                            let (id, sid) = sent_sid.take(&element).unwrap();

                            let send = nodeevent_send.send(NodeEvent::MessageSentSuccessfully(
                                messages::Message {
                                    source_id: id,
                                    session_id: sid,
                                    content: random_mtype(),
                                },
                            ));
                        }
                    }

                    let send = nodeevent_send.send(NodeEvent::MessageReceived(messages::Message {
                        source_id: id,
                        session_id: sid,
                        content: random_mtype(),
                    }));
                }
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
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
            ServerType::TextServer,
        )),
    }
}
