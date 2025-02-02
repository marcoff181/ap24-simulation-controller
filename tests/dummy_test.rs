pub mod common;

const WAITING_TIME: u64 = 300;


#[cfg(feature = "integration_tests")]
use common::start_dummy_sc_from_cfg;

#[cfg(feature = "integration_tests")]
use ap24_simulation_controller::AppMessage;

#[cfg(feature = "integration_tests")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
#[cfg(feature = "integration_tests")]
fn quit() {
    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(WAITING_TIME));
    if !sc_handle.is_finished() {
        panic!("sc is not finished 100ms after quit mesage");
    }
}

#[cfg(feature = "integration_tests")]
#[test]
fn unexpected_thread_exit() {
    use std::collections::HashMap;

    use common::start_dummy_sc_from_cfg_with_handles;

    let failed_thread = thread::spawn(move || ());
    let h = HashMap::from([(1, failed_thread)]);

    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg_with_handles("./tests/config_files/line.toml", h);
    thread::sleep(Duration::from_millis(WAITING_TIME));
    if sc_handle.is_finished() {
        match sc_handle.join() {
            Ok(_) => panic!("sim controller thread exited succesfully"),
            Err(e) => {
                println!("Thread exited with an error: {:?}", e);
            }
        }
    } else {
        panic!("sim controller thread did not exit")
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn spawn() {
    use core::panic;

    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(WAITING_TIME));
    if sc_handle.is_finished() {
        panic!("sc should still be running");
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn changepdr() {
    use core::panic;

    use common::{expect_command_hmap, expect_just_command_hmap};

    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(WAITING_TIME));

    expect_just_command_hmap(
        &command_receivers,
        1,
        &DroneCommand::SetPacketDropRate(0.03),
    );
}

#[test]
#[cfg(feature = "integration_tests")]
fn shortcut() {
    use core::panic;

    use common::{expect_command_hmap, expect_just_command_hmap, expect_just_packet_hmap};
    use wg_2024::{controller::DroneEvent, packet::Packet};

    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");

    let mut packet = Packet {
        pack_type: wg_2024::packet::PacketType::Nack(wg_2024::packet::Nack {
            fragment_index: 0,
            nack_type: wg_2024::packet::NackType::Dropped,
        }),
        routing_header: wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![1, 2, 3],
        },
        session_id: 0,
    };

    let _ = dronevent_send.send(DroneEvent::ControllerShortcut(packet.clone()));

    thread::sleep(Duration::from_millis(WAITING_TIME));

    packet.routing_header.increase_hop_index();

    expect_just_packet_hmap(&packet_receivers, 3, &packet);
}

#[test]
#[cfg(feature = "integration_tests")]
fn crash() {
    use common::{expect_command_hmap, expect_just_command_hmap};

    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(WAITING_TIME));

    expect_command_hmap(&command_receivers, 1, &DroneCommand::RemoveSender(2));
    expect_command_hmap(&command_receivers, 2, &DroneCommand::Crash);
    expect_just_command_hmap(&command_receivers, 3, &DroneCommand::RemoveSender(2));
}

#[test]
#[cfg(feature = "integration_tests")]
fn add_connection() {
    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");

    // try to connect already connected nodes
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

    let rcv1 = command_receivers.get(&1).unwrap();
    let rcv3 = command_receivers.get(&3).unwrap();

    thread::sleep(Duration::from_millis(WAITING_TIME));
    if let Ok(command) = rcv1.try_recv() {
        panic!("unexpected command : {:?}", command);
    };
    if let Ok(command) = rcv3.try_recv() {
        panic!("unexpected command : {:?}", command);
    };

    thread::sleep(Duration::from_millis(WAITING_TIME));
    //exit from error screen
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    // try to connect not already connected nodes
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(WAITING_TIME));
    let command = rcv1.try_recv().unwrap();
    if !matches!(command, DroneCommand::AddSender(3, _)) {
        panic!("unexpected command : {:?}", command);
    }
    let command = rcv3.try_recv().unwrap();
    if !matches!(command, DroneCommand::AddSender(1, _)) {
        panic!("unexpected command : {:?}", command);
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn move_node() {
    let (
        keyevent_send,
        sc_handle,
        dronevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    for x in 1..=6 {
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }
    thread::sleep(Duration::from_millis(WAITING_TIME));
    if sc_handle.is_finished() {
        panic!("sc should be still running");
    }
}
#[test]
#[cfg(feature = "integration_tests")]
fn msent_before_startingtransmission() {
    use common::all_the_message_types;
    use messages::{node_event::NodeEvent, Message, MessageType, RequestType, TextRequest};

    let (
        keyevent_send,
        sc_handle,
        droneevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");

    let mtype = MessageType::Request(RequestType::TextRequest(TextRequest::TextList));
    let x = 1;
    let session_id = 0;

    let _ = nodeevent_send.send(NodeEvent::MessageSentSuccessfully(Message {
        source: x,
        destination: x + 1,
        session_id: session_id as u64,
        content: mtype.clone(),
    }));

    let _ = nodeevent_send.send(NodeEvent::StartingMessageTransmission(Message {
        source: x,
        destination: x + 1,
        session_id: session_id as u64,
        content: mtype.clone(),
    }));

    thread::sleep(Duration::from_millis(WAITING_TIME));
    if sc_handle.is_finished() {
        match sc_handle.join() {
            Ok(_) => panic!("sim controller thread exited succesfully"),
            Err(e) => {
                println!("Thread exited with an error: {:?}", e);
            }
        }
    } else {
        panic!("sim controller thread did not exit")
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn view_packet_events() {
    use common::{all_the_message_types, all_the_packet_types, random_packet};
    use messages::{
        node_event::{EventNetworkGraph, EventNetworkNode, NodeEvent},
        Message,
    };
    use wg_2024::{
        controller::DroneEvent,
        packet::{Fragment, NodeType, Packet, PacketType},
    };

    let (
        keyevent_send,
        sc_handle,
        droneevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");

    //go through all the tabs when there is nothing inside
    for x in 1..=6 {
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        thread::sleep(Duration::from_millis(100));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        thread::sleep(Duration::from_millis(100));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        thread::sleep(Duration::from_millis(100));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        thread::sleep(Duration::from_millis(100));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        thread::sleep(Duration::from_millis(100));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }

    for x in 1..=4 {
        thread::sleep(Duration::from_millis(10));
        for ptype in all_the_packet_types(x) {
            let _ = droneevent_send.send(DroneEvent::PacketSent(Packet {
                pack_type: ptype.clone(),
                routing_header: wg_2024::network::SourceRoutingHeader {
                    hop_index: 1,
                    hops: vec![x, x + 1],
                },
                session_id: 0,
            }));
        }

        let _ = droneevent_send.send(DroneEvent::PacketDropped(Packet {
            pack_type: PacketType::MsgFragment(Fragment {
                fragment_index: 0,
                total_n_fragments: 1,
                length: 128,
                data: [35; 128],
            }),
            routing_header: wg_2024::network::SourceRoutingHeader {
                hop_index: 1,
                hops: vec![x, x + 1],
            },
            session_id: 0,
        }));
    }

    for x in 5..=6 {
        thread::sleep(Duration::from_millis(10));
        for ptype in all_the_packet_types(x) {
            let _ = nodeevent_send.send(NodeEvent::PacketSent(Packet {
                pack_type: ptype,
                routing_header: wg_2024::network::SourceRoutingHeader {
                    hop_index: 1,
                    hops: vec![x, 4],
                },
                session_id: 0,
            }));
        }

        for (session_id, mtype) in all_the_message_types().into_iter().enumerate() {
            let _ = nodeevent_send.send(NodeEvent::StartingMessageTransmission(Message {
                source: x,
                destination: x + 1,
                session_id: session_id as u64,
                content: mtype.clone(),
            }));

            let _ = nodeevent_send.send(NodeEvent::MessageSentSuccessfully(Message {
                source: x,
                destination: x + 1,
                session_id: session_id as u64,
                content: mtype.clone(),
            }));

            let _ = nodeevent_send.send(NodeEvent::MessageReceived(Message {
                source: x,
                destination: x + 1,
                session_id: session_id as u64,
                content: mtype,
            }));

            let _ = nodeevent_send.send(NodeEvent::KnownNetworkGraph {
                source: x,
                graph: EventNetworkGraph {
                    nodes: vec![
                        EventNetworkNode {
                            node_id: { 1 },
                            node_type: NodeType::Drone,
                            neighbors: vec![2, 3],
                        },
                        EventNetworkNode {
                            node_id: { 2 },
                            node_type: NodeType::Server,
                            neighbors: vec![1, 3],
                        },
                        EventNetworkNode {
                            node_id: { 3 },
                            node_type: NodeType::Client,
                            neighbors: vec![1, 2],
                        },
                    ],
                },
            });
        }
    }

    thread::sleep(Duration::from_millis(WAITING_TIME));
    //go through all the tabs after packets and messages have been sent
    for _ in 1..=6 {
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        thread::sleep(Duration::from_millis(10));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        for _ in 1..=10 {
            let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(10));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        for _ in 1..=10 {
            let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(10));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        for _ in 1..=10 {
            let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(10));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    }

    thread::sleep(Duration::from_millis(WAITING_TIME));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(1000));
    if !sc_handle.is_finished() {
        panic!("sc is not finished 100ms after quit mesage");
    }
}
