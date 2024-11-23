mod keypress_handler;
mod model;
mod utilities;
mod view;

use std::collections::HashMap;

use crate::model::Model;
use crossbeam_channel::{Receiver, Sender};
use model::screen::Screen;
use ratatui::DefaultTerminal;
use wg_2024::{
    config::Config,
    controller::{Command, SimControllerOptions, SimulationController},
    network::NodeId, packet::Packet,
};

pub struct MySimulationController {
    command_send: HashMap<NodeId, Sender<Command>>,
    command_recv: Receiver<Command>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    config: Config,
    model: Model,
}

impl SimulationController for MySimulationController {
    fn new(opt: SimControllerOptions) -> Self {
        MySimulationController {
            command_send: opt.command_send,
            command_recv: opt.command_recv,
            packet_send: opt.packet_send,
            config: opt.config.clone(),
            model: Model::new(&opt.config),
        }
    }

    // could return Result and then thread handler in network intializer handles the Error
    fn run(&mut self) {
        let terminal = ratatui::init();
        let _result = self.start(terminal);
        ratatui::restore();
    }
}

impl MySimulationController {
    fn start(&mut self, mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
        let mut running = true;
        self.model.node_list_state.select(Some(0));

        while running {
            // the view renders based on an immutable reference to the model
            // apart from that list that needed it
            terminal.draw(|frame| {
                crate::view::render(&mut self.model, frame.area(), frame.buffer_mut())
            })?;
            // keypress handler returns a Action enum or something and based on that we decide what to do
            // when the event handling requires just modifying the model it is done inside the function
            // but when there are modifications that involve SimulationController and Communication between Nodes
            // there is an AppMessage struct that comes back
            if let Some(message) = keypress_handler::handle_crossterm_events(&mut self.model)? {
                match message {
                    utilities::app_message::AppMessage::AddConnection { from, to } => {
                        self.add_connection(from, to)
                    }
                    utilities::app_message::AppMessage::Crash { drone: id } => self.crash(id),
                    utilities::app_message::AppMessage::Quit => running = false,
                }
            };
        }
        Ok(())
    }
    // handle commands from drone

    fn add_connection(&mut self, from: NodeId, to: NodeId) {
        

        // tell the real nodes via command channels to add edge
        if let (Some(command_sender_from), Some(command_sender_to), Some(packet_sender_to),Some(packet_sender_from)) = (
            self.command_send.get(&from),
            self.command_send.get(&to),
            self.packet_send.get(&from),
            self.packet_send.get(&to),
        ) {
            command_sender_from.send(Command::AddChannel(to, packet_sender_to.clone()));
            command_sender_to.send(Command::AddChannel(from,packet_sender_from.clone()));

            // for now we assume they succesfully added channel, and show it in the model
            self.model.add_edge(from, to);
            self.model.select_node(from);
            self.model.screen = Screen::Main;
        }
    }

    fn crash(&mut self, id: NodeId) {
        // send command to corresponding drone to crash
        if let Some(drone_command_sender) = self.command_send.get(&id) {
            // todo: handle error
            let _ = drone_command_sender.send(Command::Crash);
        }

        // set in the model the corresponding node to crashed true
        self.model.crash_drone(id);
    }
}
