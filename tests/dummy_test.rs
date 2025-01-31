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
    let (keyevent_send, sc_handle, dronevent_send, nodeevent_send, command_receivers) =
        start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));
    if !sc_handle.is_finished() {
        panic!("sc is not finished 100ms after quit mesage");
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn spawn() {
    use core::panic;

    let (keyevent_send, sc_handle, dronevent_send, nodeevent_send, command_receivers) =
        start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));
    if sc_handle.is_finished() {
        panic!("sc should still be running");
    }
}

#[test]
#[cfg(feature = "integration_tests")]
fn changepdr() {
    use core::panic;

    use common::{expect_command_hmap, expect_just_command_hmap};

    let (keyevent_send, sc_handle, dronevent_send, nodeevent_send, command_receivers) =
        start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));

    expect_just_command_hmap(
        &command_receivers,
        1,
        &DroneCommand::SetPacketDropRate(0.03),
    );
}

#[test]
#[cfg(feature = "integration_tests")]
fn crash() {
    use common::{expect_command_hmap, expect_just_command_hmap};

    let (keyevent_send, sc_handle, dronevent_send, nodeevent_send, command_receivers) =
        start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));

    expect_command_hmap(&command_receivers, 1, &DroneCommand::RemoveSender(2));
    expect_command_hmap(&command_receivers, 2, &DroneCommand::Crash);
    expect_just_command_hmap(&command_receivers, 3, &DroneCommand::RemoveSender(2));
}

#[test]
#[cfg(feature = "integration_tests")]
fn add_connection() {
    let (keyevent_send, sc_handle, dronevent_send, nodeevent_send, command_receivers) =
        start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    thread::sleep(Duration::from_millis(100));
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
    let (keyevent_send, sc_handle, dronevent_send, nodeevent_send, command_receivers) =
        start_dummy_sc_from_cfg("./tests/config_files/line.toml");
    for x in 1..=6 {
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let _ = keyevent_send.send(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }
    thread::sleep(Duration::from_millis(100));
    if sc_handle.is_finished() {
        panic!("sc should be still running");
    }
}
