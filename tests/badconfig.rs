pub mod common;
const WAITING_TIME: u64 = 300;

#[cfg(feature = "integration_tests")]
use common::start_dummy_sc_from_cfg;
use std::{thread, time::Duration};

#[cfg(feature = "integration_tests")]
#[test]
#[should_panic]
fn disconnected() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/disconnected.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}

#[cfg(feature = "integration_tests")]
#[test]
#[should_panic]
fn clientcenter() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/clientcenter.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}

#[cfg(feature = "integration_tests")]
#[test]
#[should_panic(
    expected = "when converting cfg to network found error: Server must connect to at least two drones"
)]
fn server_one_connection() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/server_one_connection.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}

#[cfg(feature = "integration_tests")]
#[test]
#[should_panic(
    expected = "when converting cfg to network found error: Client must connect to at least one and at most two drones"
)]
fn client_connected_3() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/client_connected_3.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}

#[cfg(feature = "integration_tests")]
#[test]
#[should_panic]
fn sameid() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/sameid.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}

#[cfg(feature = "integration_tests")]
#[test]
#[should_panic]
fn neighbor_is_self() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/neighbor_is_self.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}

#[cfg(feature = "integration_tests")]
#[test]
fn nopanic() {
    let _ = start_dummy_sc_from_cfg("./tests/config_files/input.toml");
    thread::sleep(Duration::from_millis(WAITING_TIME));
}
