use std::{collections::HashMap, task::Wake, thread, time::Duration};

use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
use crossbeam_channel::{self, unbounded, Receiver, Sender};
use log::LevelFilter;
use messages::node_event::NodeEvent;
use simplelog::{format_description, ConfigBuilder, ThreadLogMode, ThreadPadding, WriteLogger};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, NackType, Packet, PacketType},
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
        nodeevent_send,
        packet_send: packet_senders,
        config,
        node_handles: HashMap::new(),
    };
    let mut simcontr = MySimulationController::new(opt);
    let join_handle = thread::spawn(move || {
        simcontr.run();
    });
    loop {
        if join_handle.is_finished() {
            println!("Simulation Controller has finished its work.");
            break;
        }

        let send = droneevent_send.send(DroneEvent::PacketSent(random_packet()));
        let send = droneevent_send.send(DroneEvent::PacketDropped(Packet {
            pack_type: PacketType::MsgFragment(Fragment {
                fragment_index: rand::random_range(1..256),
                total_n_fragments: rand::random_range(1..345),
                length: rand::random_range(1..128),
                data: [35; 128],
            }),
            routing_header: wg_2024::network::SourceRoutingHeader {
                hop_index: rand::random_range(1..=6) as usize,
                hops: vec![1, 2, 3, 4, 5, 6],
            },
            session_id: rand::random_range(1..256),
        }));

        thread::sleep(Duration::from_millis(100));
    }
}
pub fn random_packet() -> Packet {
    Packet {
        pack_type: match rand::random_range(1..=5u64) {
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
                initiator_id: 4,
                path_trace: vec![],
            }),
            _ => PacketType::FloodResponse(FloodResponse {
                flood_id: 2,
                path_trace: vec![],
            }),
        },
        routing_header: wg_2024::network::SourceRoutingHeader {
            hop_index: rand::random_range(1..=6) as usize,
            hops: vec![1, 2, 3, 4, 5, 6],
        },
        session_id: rand::random_range(1..256),
    }
}
