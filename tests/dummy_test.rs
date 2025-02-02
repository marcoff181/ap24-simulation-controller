pub mod common;

use wg_2024::controller::DroneCommand;

#[cfg(feature = "integration_tests")]
use common::start_dummy_sc_from_cfg;
use std::{thread, time::Duration};
use test_log::test;

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
    thread::sleep(Duration::from_millis(100));
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
    thread::sleep(Duration::from_millis(200));
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
    thread::sleep(Duration::from_millis(200));
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
    thread::sleep(Duration::from_millis(200));

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

    thread::sleep(Duration::from_millis(200));

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
    thread::sleep(Duration::from_millis(200));

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
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(200));
    let rcv = command_receivers.get(&1).unwrap();
    let command = rcv.try_recv().unwrap();
    if !matches!(command, DroneCommand::AddSender(3, _)) {
        panic!("unexpected command : {:?}", command);
    }
    let rcv = command_receivers.get(&3).unwrap();
    let command = rcv.try_recv().unwrap();
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
    thread::sleep(Duration::from_millis(200));
    if sc_handle.is_finished() {
        panic!("sc should be still running");
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn view_packet_events() {
    use common::random_packet;
    use wg_2024::{
        controller::DroneEvent,
        packet::{Fragment, Packet, PacketType},
    };

    let (
        keyevent_send,
        sc_handle,
        droneevent_send,
        nodeevent_send,
        command_receivers,
        _packet_receivers,
    ) = start_dummy_sc_from_cfg("./tests/config_files/line.toml");

    let _ = droneevent_send.send(DroneEvent::PacketSent(Packet {
        pack_type: random_packet(),
        routing_header: wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![1, 2, 3],
        },
        session_id: 0,
    }));
    let _ = droneevent_send.send(DroneEvent::PacketDropped(Packet {
        pack_type: PacketType::MsgFragment(Fragment {
            fragment_index: rand::random_range(1..256),
            total_n_fragments: rand::random_range(1..345),
            length: rand::random_range(1..128),
            data: [35; 128],
        }),
        routing_header: wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![1, 2, 3],
        },
        session_id: 0,
    }));

    thread::sleep(Duration::from_millis(100));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));
    if !sc_handle.is_finished() {
        panic!("sc is not finished 100ms after quit mesage");
    }
}

//                let send = nodeevent_send.send(NodeEvent::PacketSent(Packet {
//                    pack_type: random_packet(),
//                    routing_header: wg_2024::network::SourceRoutingHeader {
//                        hop_index: 3,
//                        hops: vec![2, 3, id, rand::random_range(1..=6)],
//                    },
//                    session_id: sid,
//                }));
//                if rand::random_bool(0.4) {
//                    let send = nodeevent_send.send(NodeEvent::StartingMessageTransmission(
//                        messages::Message {
//                            source_id: id,
//                            session_id: sid,
//                            content: random_mtype(),
//                        },
//                    ));
//                    sent_sid.insert((id, sid));
//                }
//                if rand::random_bool(0.3) {
//                    if let Some(&element) = sent_sid.iter().next() {
//                        let (id, sid) = sent_sid.take(&element).unwrap();
//
//                        let send = nodeevent_send.send(NodeEvent::MessageSentSuccessfully(
//                            messages::Message {
//                                source_id: id,
//                                session_id: sid,
//                                content: random_mtype(),
//                            },
//                        ));
//                    }
//                }
//
//                let send = nodeevent_send.send(NodeEvent::MessageReceived(messages::Message {
//                    source_id: id,
//                    session_id: sid,
//                    content: random_mtype(),
//                }));
