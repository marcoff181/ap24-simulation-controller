use std::collections::HashMap;

use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
use wg_2024::{config::Config, controller::{Command, SimulationController}, network::NodeId, packet::Packet};
use crossbeam_channel::{self, unbounded, Receiver, Sender};

// used while developing to check how the GUI is functioning
fn main(){
    let config_data = std::fs::read_to_string("./src/tests/config_files/input.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    
    let (_dummy_command_to_simcontr,command_from_node) = unbounded::<Command>();
    
    let mut dummy_drone_receivers :HashMap<NodeId,Receiver<Command>> = HashMap::new();
    let mut simcontroller_senders : HashMap<NodeId,Sender<Command>> = HashMap::new();

    for n in config.drone.iter(){
        let (command_to_node,command_from_simcontr) = unbounded::<Command>();
        dummy_drone_receivers.insert(n.id, command_from_simcontr);
        simcontroller_senders.insert(n.id, command_to_node);
    }

    for n in config.server.iter(){
        let (command_to_node,command_from_simcontr) = unbounded::<Command>();
        dummy_drone_receivers.insert(n.id, command_from_simcontr);
        simcontroller_senders.insert(n.id, command_to_node);
    }

    for n in config.client.iter(){
        let (command_to_node,command_from_simcontr) = unbounded::<Command>();
        dummy_drone_receivers.insert(n.id, command_from_simcontr);
        simcontroller_senders.insert(n.id, command_to_node);
    }

    let opt = SimControllerOptions{
        command_send: simcontroller_senders,
        command_recv: command_from_node,
        // todo: simulate this too
        packet_send:HashMap::<NodeId,Sender<Packet>>::new(),
        config: config,
    };

    let mut simcontr = MySimulationController::new(opt);
    simcontr.run();

    // here you can do something with dummy_command_to_stimcontr and dummy_drone_receivers to check if 
}