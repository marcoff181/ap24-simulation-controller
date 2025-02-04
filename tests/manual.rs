//use crossbeam_channel::unbounded;
//use crossbeam_channel::Sender;
//use crossbeam_channel::{self, Receiver};
//use messages::node_event::NodeEvent;
//use messages::{
//    ChatRequest, ChatResponse, ErrorType, MediaRequest, MediaResponse, MessageType, RequestType,
//    ResponseType, ServerType, TextRequest, TextResponse,
//};
//use ratatui::{backend::TestBackend, Terminal};
//use std::collections::HashMap;
//use std::thread;
//use std::thread::JoinHandle;
//use wg_2024::config::Config;
//use wg_2024::controller::DroneEvent;
//use wg_2024::{
//    controller::DroneCommand,
//    network::NodeId,
//    packet::{
//        Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType,
//    },
//};
//
//use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
//use crossterm::event::KeyEvent;
//use std::time::Duration;
//
//fn start_dummy_sc_from_cfg(
//    config: &str,
//) -> (
//    Sender<KeyEvent>,
//    JoinHandle<()>,
//    Sender<DroneEvent>,
//    Sender<NodeEvent>,
//    HashMap<NodeId, Receiver<DroneCommand>>,
//    HashMap<NodeId, Receiver<Packet>>,
//) {
//    start_dummy_sc_from_cfg_with_handles(config, HashMap::new())
//}
//
//fn start_dummy_sc_from_cfg_with_handles(
//    config: &str,
//    mut node_handles: HashMap<u8, JoinHandle<()>>,
//) -> (
//    Sender<KeyEvent>,
//    JoinHandle<()>,
//    Sender<DroneEvent>,
//    Sender<NodeEvent>,
//    HashMap<NodeId, Receiver<DroneCommand>>,
//    HashMap<NodeId, Receiver<Packet>>,
//) {
//    let add_handles = node_handles.is_empty();
//
//    let config_data = std::fs::read_to_string(config).expect("Unable to read config file");
//    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
//
//    let (droneevent_send, droneevent_recv) = unbounded::<DroneEvent>();
//    let (nodeevent_send, nodeevent_recv) = unbounded::<NodeEvent>();
//    let (appmess_send, keyevent_recv) = unbounded::<KeyEvent>();
//
//    let mut command_receivers: HashMap<NodeId, Receiver<DroneCommand>> = HashMap::new();
//    let mut command_senders: HashMap<NodeId, Sender<DroneCommand>> = HashMap::new();
//    let mut packet_receivers: HashMap<NodeId, Receiver<Packet>> = HashMap::new();
//    let mut packet_senders: HashMap<NodeId, Sender<Packet>> = HashMap::new();
//
//    for n in config.drone.iter() {
//        let (cs, cr) = unbounded::<DroneCommand>();
//        command_receivers.insert(n.id, cr);
//        command_senders.insert(n.id, cs);
//        let (ps, pr) = unbounded::<Packet>();
//        packet_receivers.insert(n.id, pr);
//        packet_senders.insert(n.id, ps);
//        if add_handles {
//            let handle = thread::Builder::new()
//                .name(format!("drone#{}", n.id))
//                .spawn(move || loop {
//                    thread::sleep(Duration::from_secs(1));
//                })
//                .expect("could not create thread");
//            node_handles.insert(n.id, handle);
//        }
//    }
//
//    for n in config.server.iter() {
//        let (cs, cr) = unbounded::<DroneCommand>();
//        command_receivers.insert(n.id, cr);
//        command_senders.insert(n.id, cs);
//        let (ps, pr) = unbounded::<Packet>();
//        packet_receivers.insert(n.id, pr);
//        packet_senders.insert(n.id, ps);
//        if add_handles {
//            let handle = thread::Builder::new()
//                .name(format!("server#{}", n.id))
//                .spawn(move || loop {
//                    thread::sleep(Duration::from_secs(1));
//                })
//                .expect("could not create thread");
//            node_handles.insert(n.id, handle);
//        }
//    }
//
//    for n in config.client.iter() {
//        let (cs, cr) = unbounded::<DroneCommand>();
//        command_receivers.insert(n.id, cr);
//        command_senders.insert(n.id, cs);
//        let (ps, pr) = unbounded::<Packet>();
//        packet_receivers.insert(n.id, pr);
//        packet_senders.insert(n.id, ps);
//        if add_handles {
//            let handle = thread::Builder::new()
//                .name(format!("client#{}", n.id))
//                .spawn(move || loop {
//                    thread::sleep(Duration::from_secs(1));
//                })
//                .expect("could not create thread");
//            node_handles.insert(n.id, handle);
//        }
//    }
//
//    let opt = SimControllerOptions {
//        command_send: command_senders,
//        droneevent_recv,
//        droneevent_send: droneevent_send.clone(),
//        nodeevent_recv,
//        packet_send: packet_senders,
//        config,
//        node_handles,
//    };
//
//    let terminal = Terminal::new(TestBackend::new(50, 50)).unwrap();
//    let mut simcontr = MySimulationController::new(opt);
//    //simcontr.set_keyevent_recv(keyevent_recv);
//    let join_handle = thread::spawn(move || {
//        simcontr.run();
//        //simcontr.run_with_terminal(terminal);
//    });
//    (
//        appmess_send,
//        join_handle,
//        droneevent_send,
//        nodeevent_send,
//        command_receivers,
//        packet_receivers,
//    )
//}
//
//#[test]
//fn abc() {
//    let (
//        keyevent_send,
//        sc_handle,
//        dronevent_send,
//        nodeevent_send,
//        command_receivers,
//        _packet_receivers,
//    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");
//
//    thread::sleep(Duration::from_millis(2000000));
//}
