//use std::{
//    collections::{HashMap, HashSet},
//    task::Wake,
//    thread,
//    time::Duration,
//};
//
//use ap24_simulation_controller::{MySimulationController, SimControllerOptions};
//use crossbeam_channel::{self, unbounded, Receiver, Sender};
//use log::LevelFilter;
//use messages::{node_event::NodeEvent, MediaRequest, MessageType, ServerType};
//use simplelog::{format_description, ConfigBuilder, ThreadLogMode, ThreadPadding, WriteLogger};
//use wg_2024::{
//    config::Config,
//    controller::{DroneCommand, DroneEvent},
//    network::NodeId,
//    packet::{Ack, FloodRequest, FloodResponse, Fragment, NackType, NodeType, Packet, PacketType},
//};
//
//// used while developing to check how the GUI is functioning

fn main() {}
//fn main() {
//    let log_level = LevelFilter::Debug;
//    let _logger = WriteLogger::init(
//        log_level,
//        ConfigBuilder::new()
//            //.set_thread_mode(ThreadLogMode::Both)
//            .set_thread_level(LevelFilter::Error)
//            //.set_thread_padding(ThreadPadding::Right(15))
//            .build(),
//        std::fs::File::create("app.log").expect("Could not create log file"),
//    );
//
//    //let mut sent_sid = HashSet::new();
//    //let mut sid = 0;
//    //loop {
//    //    sid += 1;
//    //    if join_handle.is_finished() {
//    //        println!("Simulation Controller has finished its work.");
//    //        break;
//    //    }
//    //
//    //    for id in 1..=6u8 {
//    //        match id {
//    //            1..=3 => {
//    //                let send = droneevent_send.send(DroneEvent::PacketSent(Packet {
//    //                    pack_type: random_packet(),
//    //                    routing_header: wg_2024::network::SourceRoutingHeader {
//    //                        hop_index: 3,
//    //                        hops: vec![2, 3, id, rand::random_range(1..=6)],
//    //                    },
//    //                    session_id: sid,
//    //                }));
//    //                let send = droneevent_send.send(DroneEvent::PacketDropped(Packet {
//    //                    pack_type: PacketType::MsgFragment(Fragment {
//    //                        fragment_index: rand::random_range(1..256),
//    //                        total_n_fragments: rand::random_range(1..345),
//    //                        length: rand::random_range(1..128),
//    //                        data: [35; 128],
//    //                    }),
//    //                    routing_header: wg_2024::network::SourceRoutingHeader {
//    //                        hop_index: 3,
//    //                        hops: vec![2, 3, id, rand::random_range(1..=6)],
//    //                    },
//    //                    session_id: sid,
//    //                }));
//    //            }
//    //            _ => {
//    //                let send = nodeevent_send.send(NodeEvent::PacketSent(Packet {
//    //                    pack_type: random_packet(),
//    //                    routing_header: wg_2024::network::SourceRoutingHeader {
//    //                        hop_index: 3,
//    //                        hops: vec![2, 3, id, rand::random_range(1..=6)],
//    //                    },
//    //                    session_id: sid,
//    //                }));
//    //                if rand::random_bool(0.4) {
//    //                    let send = nodeevent_send.send(NodeEvent::StartingMessageTransmission(
//    //                        messages::Message {
//    //                            source_id: id,
//    //                            session_id: sid,
//    //                            content: random_mtype(),
//    //                        },
//    //                    ));
//    //                    sent_sid.insert((id, sid));
//    //                }
//    //                if rand::random_bool(0.3) {
//    //                    if let Some(&element) = sent_sid.iter().next() {
//    //                        let (id, sid) = sent_sid.take(&element).unwrap();
//    //
//    //                        let send = nodeevent_send.send(NodeEvent::MessageSentSuccessfully(
//    //                            messages::Message {
//    //                                source_id: id,
//    //                                session_id: sid,
//    //                                content: random_mtype(),
//    //                            },
//    //                        ));
//    //                    }
//    //                }
//    //
//    //                let send = nodeevent_send.send(NodeEvent::MessageReceived(messages::Message {
//    //                    source_id: id,
//    //                    session_id: sid,
//    //                    content: random_mtype(),
//    //                }));
//    //            }
//    //        }
//    //    }
//    //
//    thread::sleep(Duration::from_millis(500));
//    //}
//}
