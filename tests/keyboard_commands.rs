use core::panic;
use std::{collections::HashMap, sync::mpsc, task::Wake, thread, time::Duration};

use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
use crossbeam_channel::{self, unbounded, Receiver, Sender};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, NackType, Packet, PacketType},
};
// used while developing to check how the GUI is functioning

#[test]
fn changes() {
    let config_data = std::fs::read_to_string("./tests/config_files/input.toml")
        .expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    let (dummy_command_to_simcontr, event_from_node) = unbounded::<DroneEvent>();

    let mut dummy_drone_receivers: HashMap<NodeId, Receiver<DroneCommand>> = HashMap::new();
    let mut simcontroller_senders: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();

    for n in config.drone.iter() {
        let (command_to_node, command_from_simcontr) = unbounded::<DroneCommand>();
        dummy_drone_receivers.insert(n.id, command_from_simcontr);
        simcontroller_senders.insert(n.id, command_to_node);
    }

    for n in config.server.iter() {
        let (command_to_node, command_from_simcontr) = unbounded::<DroneCommand>();
        dummy_drone_receivers.insert(n.id, command_from_simcontr);
        simcontroller_senders.insert(n.id, command_to_node);
    }

    for n in config.client.iter() {
        let (command_to_node, command_from_simcontr) = unbounded::<DroneCommand>();
        dummy_drone_receivers.insert(n.id, command_from_simcontr);
        simcontroller_senders.insert(n.id, command_to_node);
    }

    let opt = SimControllerOptions {
        command_send: simcontroller_senders,
        event_recv: event_from_node,
        // todo: simulate this too
        packet_send: HashMap::<NodeId, Sender<Packet>>::new(),
        config,
        node_handles: Vec::new(),
    };
    let mut simcontr = MySimulationController::new(opt);
    let join_handle = thread::spawn(move || {
        simcontr.run();
    });

    let (tx, rx) = mpsc::channel();
    thread::sleep(Duration::from_millis(500));
    // Simulate pressing the 'q' key
    tx.send(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE))
        .unwrap();
    // Add a small delay to mimic user behavior
    thread::sleep(Duration::from_millis(100));

    if join_handle.is_finished() {
        println!("Simulation Controller has finished its work.");
    } else {
        panic!("sc did not exit");
    }

    //let send = dummy_command_to_simcontr.send(DroneEvent::PacketSent(random_packet()));
    //let send = dummy_command_to_simcontr.send(DroneEvent::PacketDropped(Packet {
    //    pack_type: PacketType::MsgFragment(Fragment {
    //        fragment_index: rand::random_range(1..256),
    //        total_n_fragments: rand::random_range(1..345),
    //        length: rand::random_range(1..128),
    //        data: [35; 128],
    //    }),
    //    routing_header: wg_2024::network::SourceRoutingHeader {
    //        hop_index: rand::random_range(1..=6) as usize,
    //        hops: vec![1, 2, 3, 4, 5, 6],
    //    },
    //    session_id: rand::random_range(1..256),
    //}));
    //
    //thread::sleep(Duration::from_millis(100));
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
