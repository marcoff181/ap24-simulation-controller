use crate::network;
use crate::screen;
use crate::utilities;
use crate::MySimulationController;

use log::{debug, info, trace};
use network::node_kind::NodeKind;
use screen::Window;
use utilities::app_message::AppMessage;

impl MySimulationController {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn transition(&mut self, message: &AppMessage) {
        let kind = self.screen.kind;
        let id = self.screen.focus;
        match message {
            AppMessage::Quit => {
                info!("received AppMessage::Quit, exiting...");
                self.running = false;
            }
            AppMessage::Crash => match self.screen.window {
                Window::Detail { tab: _ } if matches!(kind, NodeKind::Drone { .. }) => {
                    match self.crash(id) {
                        Ok(()) => {
                            self.screen.window = Window::Main;
                        }
                        Err(message) => {
                            debug!("error crashing drone, switching to Window::Error");
                            self.screen.window = Window::Error { message };
                        }
                    };
                }
                _ => {}
            },
            // for Detail
            AppMessage::ChangeTab => {
                if let Window::Detail { ref mut tab } = self.screen.window {
                    *tab = tab.saturating_add(1);
                    self.packet_table_state.select(Some(0));
                    *tab %= 3;
                    trace!(
                        "On window Detail of #{}, switched to tab {tab}",
                        self.screen.focus
                    );
                    //if let NodeKind::Drone { .. } = kind {
                    //    *tab %= 3;
                    //} else {
                    //    *tab %= 3;
                    //}
                }
            }
            // spawn drone
            AppMessage::SpawnDrone => {
                if let Window::Main = self.screen.window {
                    self.spawn_drone();
                }
            }
            // Window changes
            AppMessage::WindowAddConnection => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::AddConnection { origin: id }
                }
            }
            AppMessage::WindowChangePDR => {
                if let Window::Detail { tab: _ } = self.screen.window {
                    if let NodeKind::Drone {
                        pdr,
                        crashed: false,
                    } = kind
                    {
                        self.screen.window = Window::ChangePdr { pdr }
                    }
                }
            }
            AppMessage::WindowMove => {
                if let Window::Main = self.screen.window {
                    self.screen.window = Window::Move;
                }
            }
            AppMessage::WindowDetail => {
                if let Window::Main = self.screen.window {
                    self.packet_table_state.select_first();
                    self.screen.window = Window::Detail { tab: 0 }
                }
            }
            AppMessage::Done => match self.screen.window {
                Window::Main => {}
                Window::Error { message: _ } => {
                    self.reset_list();
                    self.screen.window = Window::Main;
                }
                Window::Move | Window::Detail { tab: _ } => self.screen.window = Window::Main,
                Window::AddConnection { origin } => {
                    info!("received AppMessage::Done, current window is AddConnection, adding connection...");
                    match self.add_connection(origin, id) {
                        Ok(()) => {
                            self.reset_list();
                            self.screen.window = Window::Main;
                            info!("connection added succesfully, switched back to Window::Main");
                        }
                        Err(s) => {
                            debug!("could not add connection, switching to Window::Error");
                            self.screen.window = Window::Error { message: s };
                        }
                    };
                }
                Window::ChangePdr { pdr } => {
                    self.change_pdr(pdr);
                    self.screen.window = Window::Detail { tab: 0 };
                }
            },
            // List movement
            AppMessage::ScrollUp => match self.screen.window {
                Window::Main | Window::AddConnection { .. } => {
                    self.scroll_list(true);
                }
                Window::Detail { .. } => {
                    self.packet_table_state.scroll_up_by(1);
                }
                Window::ChangePdr { ref mut pdr } => {
                    *pdr += 0.01;
                    if *pdr > 1.0 {
                        *pdr = 1.0;
                    }
                }
                _ => {}
            },
            AppMessage::ScrollDown => match self.screen.window {
                Window::Main | Window::AddConnection { .. } => {
                    self.scroll_list(false);
                }
                Window::Detail { .. } => {
                    self.packet_table_state.scroll_down_by(1);
                }
                Window::ChangePdr { ref mut pdr } => {
                    *pdr -= 0.01;
                    if *pdr < 0.0 {
                        *pdr = 0.0;
                    }
                }
                _ => {}
            },
            // Node movement
            AppMessage::MoveNode { x, y } => {
                let node = self.network.get_mut_node_from_id(id).unwrap();
                if *x > 0 {
                    node.shiftr(u32::from(x.unsigned_abs()));
                } else {
                    node.shiftl(u32::from(x.unsigned_abs()));
                }
                if *y > 0 {
                    node.shiftu(u32::from(y.unsigned_abs()));
                } else {
                    node.shiftd(u32::from(y.unsigned_abs()));
                }
            }
        }
    }
}
