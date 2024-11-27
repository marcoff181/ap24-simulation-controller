use std::{collections::HashMap, thread, time::Duration};

use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
use crossbeam_channel::{self, unbounded, Receiver, Sender};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, NodeEvent},
    network::NodeId,
    packet::Packet,
};

// used while developing to check how the GUI is functioning
fn main() {
    let config_data = std::fs::read_to_string("./src/tests/config_files/input.toml")
        .expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");

    let (dummy_command_to_simcontr, event_from_node) = unbounded::<NodeEvent>();

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

    // here you can do something with dummy_command_to_stimcontr and dummy_drone_receivers to check if
    let send = dummy_command_to_simcontr.send(NodeEvent::PacketSent(Packet {
        pack_type: wg_2024::packet::PacketType::Nack(wg_2024::packet::Nack::Dropped(35)),
        routing_header: wg_2024::network::SourceRoutingHeader {
            hop_index: 3,
            hops: vec![1, 2, 3, 4, 5, 6],
        },
        session_id: 6,
    }));
    let res = join_handle.join();
}
