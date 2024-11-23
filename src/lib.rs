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
        self.model.running = true;
        self.model.node_list_state.select(Some(0));

        while self.model.running {
            // the view renders based on an immutable reference to the model
            // apart from that list that needed it
            terminal.draw(|frame| {
                crate::view::render(&mut self.model, frame.area(), frame.buffer_mut())
            })?;
            // should keypress return something when an action is asked?
            keypress_handler::handle_crossterm_events(self)?;
        }
        Ok(())
    }
    // handle commands from drone

    fn add_connection(&mut self, from: usize, to: usize) {
        if from < self.model.nodes.len() && to < self.model.nodes.len() {
            self.model.nodes[from].adj.push(to as NodeId);
            self.model.nodes[to].adj.push(from as NodeId);
        }

        // tell the real nodes via command channels to add edge
    }

    fn crash(drone: NodeId) {
        // set in the model the corresponding node to crashed true
        todo!();
        // send command to corresponding drone to crash
    }
}
