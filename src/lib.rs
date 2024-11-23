mod keypress_handler;
mod model;
mod utilities;
mod view;

use std::collections::HashMap;

use crate::model::Model;
use crossbeam_channel::{Receiver, Sender};
use ratatui::DefaultTerminal;
use wg_2024::{
    config::Config,
    controller::{Command, SimControllerOptions, SimulationController},
    network::NodeId,
};

pub struct MySimulationController {
    command_send: HashMap<NodeId, Sender<Command>>,
    command_recv: Receiver<Command>,
    config: Config,
    model: Model,
}

impl SimulationController for MySimulationController {
    fn new(opt: SimControllerOptions) -> Self {
        MySimulationController {
            command_send: opt.command_send,
            command_recv: opt.command_recv,
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
            match keypress_handler::handle_crossterm_events(&mut self.model)? {
                Some(message) => match message{
                    utilities::app_message::AppMessage::AddConnection { from, to } => todo!(),
                    utilities::app_message::AppMessage::Crash { drone } => todo!(),
                    utilities::app_message::AppMessage::Quit => running=false,
                },
                None => {}
            };
        }
        Ok(())
    }
    // handle commands from drone

    fn add_connection(&mut self, from: usize, to: usize) {
        if from < self.model.nodes.len() && to < self.model.nodes.len() {
            self.model.nodes[from].adj.push(to as NodeId);
            self.model.nodes[to].adj.push(from as NodeId);
        }

        // model.add_connection(from as usize, x);
        //     model.node_list_state.select(Some(from as usize));
        //     model.screen = Screen::Main

        // tell the real nodes via command channels to add edge
    }

    fn crash(drone: NodeId) {
        // set in the model the corresponding node to crashed true
        todo!();
        // send command to corresponding drone to crash
    }
}
