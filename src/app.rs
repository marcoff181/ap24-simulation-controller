use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::Rng;
use ratatui::buffer::Buffer;
use ratatui::prelude::Stylize;
use ratatui::prelude::*;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Circle, Context, Painter, Shape};
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListDirection, ListState, Paragraph,
};
use ratatui::DefaultTerminal;
use std::collections::HashSet;
use std::fmt::format;
use std::io::Lines;
use tui_big_text::{BigText, PixelSize};

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..=255);
    let g = rng.gen_range(0..=255);
    let b = rng.gen_range(0..=255);

    // Combine into a u32 in the format 0x00RRGGBB
    let color_value = (r << 16) | (g << 8) | b; // 0x00RRGGBB
    Color::from_u32(color_value)
}

const BG_COLOR: Color = Color::Black;
const BOTTOMPANEL_BG: Color = Color::Black;
const TEXT_COLOR: Color = Color::White;
const INVERTED_TEXT_COLOR: Color = Color::Black;
const HIGHLIGHT_COLOR: Color = Color::LightYellow;
const DRONE_COLOR: Color = Color::LightBlue;
const CLIENT_COLOR: Color = Color::LightGreen;
const SERVER_COLOR: Color = Color::LightMagenta;

#[derive(Debug, Default)]
enum Screen {
    #[default]
    Start,
    Main,
    Move,
}

#[derive(Debug, Default, Eq, PartialEq, Hash)]
enum NodeKind {
    #[default]
    Drone,
    Client,
    Server,
}

#[derive(Debug, Default, Eq, PartialEq, Hash)]
struct Node {
    id: u32,
    x: u32,
    y: u32,
    kind: NodeKind,
    repr: String,
}

impl Node {
    fn new(id: u32, x: u32, y: u32, kind: NodeKind) -> Self {
        let s = format!("{:?} #{}", kind, id);
        Node {
            id,
            x,
            y,
            kind,
            repr: s,
        }
    }

    fn shiftr(&mut self, offset: u32) {
        self.x = self.x.saturating_add(offset);
    }

    fn shiftl(&mut self, offset: u32) {
        self.x = self.x.saturating_sub(offset);
    }

    fn shiftu(&mut self, offset: u32) {
        self.y = self.y.saturating_add(offset);
    }

    fn shiftd(&mut self, offset: u32) {
        self.y = self.y.saturating_sub(offset);
    }
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    screen: Screen,
    nodes: Vec<Node>,
    //edges: Vec<Edge>,
    node_list_state: ListState,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            node_list_state: ListState::default(),
            ..Self::default()
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), std::io::Error> {
        self.running = true;
        self.node_list_state.select(Some(0));
        self.nodes = vec![
            Node::new(1234, 0, 6, NodeKind::Drone),
            Node::new(3252, 4, 9, NodeKind::Drone),
            Node::new(6234, 7, 10, NodeKind::Drone),
            Node::new(5463, 9, 11, NodeKind::Drone),
            Node::new(5234, 2, 2, NodeKind::Drone),
            Node::new(4252, 8, 3, NodeKind::Drone),
            Node::new(8234, 1, 1, NodeKind::Drone),
            Node::new(9456, 15, 4, NodeKind::Drone),
            Node::new(3452, 3, 3, NodeKind::Client),
            Node::new(5323, 5, 4, NodeKind::Client),
            Node::new(7345, 10, 0, NodeKind::Server),
            Node::new(8945, 7, 3, NodeKind::Server),
        ];
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<(), std::io::Error> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Start => self.handle_keypress_start(key),
            Screen::Main => self.handle_keypress_main(key),
            Screen::Move => self.handle_keypress_move(key),
        }
    }

    fn handle_keypress_start(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q')) => self.quit(),
            // (_, KeyCode::Up) => self.node_list_state.scroll_up_by(1),
            // (_, KeyCode::Down) => self.node_list_state.scroll_down_by(1),
            (_, KeyCode::Char('+')) => self.screen = Screen::Main,
            _ => {}
        }
    }

    fn handle_keypress_main(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q')) => self.quit(),
            (_, KeyCode::Char('m')) => self.screen = Screen::Move,
            (_, KeyCode::Up) => self.node_list_state.scroll_up_by(1),
            (_, KeyCode::Down) => self.node_list_state.scroll_down_by(1),
            // | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C'))
            // (_,KeyCode::Char('c')) => self.quit(),
            _ => {}
        }
    }

    fn handle_keypress_move(&mut self, key: KeyEvent) {
        let n = match self.node_list_state.selected() {
            None => {
                self.screen = Screen::Main;
                return;
            }
            Some(x) => &mut self.nodes[x],
        };

        match (key.modifiers, key.code) {
            (_, KeyCode::Char('q')) => self.quit(),
            (_, KeyCode::Up) => n.shiftu(1),
            (_, KeyCode::Down) => n.shiftd(1),
            (_, KeyCode::Left) => n.shiftl(1),
            (_, KeyCode::Right) => n.shiftr(1),
            (_, KeyCode::Enter) => self.screen = Screen::Main,
            // | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C'))
            // (_,KeyCode::Char('c')) => self.quit(),
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn open_initialization_file(&mut self) {
        todo!()
    }

    fn render_start(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            // .title("Simulation Controller")
            .bg(BG_COLOR)
            .fg(TEXT_COLOR);

        let big_text = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Sextant)
            .style(Style::new().blue().bg(BG_COLOR))
            .lines(vec![
                "Simulation".red().into(),
                "Controller".white().into(),
                // "by marcoff181".into(),
            ])
            .build();

        // Get the inner area of the block to render the BigText
        let inner_area = block.inner(area);

        let [top, bottom] =
            Layout::vertical([Constraint::Max(6), Constraint::Fill(1)]).areas(inner_area);

        // Render the BigText inside the inner area
        big_text.render(top, buf);
        block.render(area, buf);
        //p.render(bottom,buf);
    }

    fn render_main(&mut self, area: Rect, buf: &mut Buffer) {
        let [left, right] =
            Layout::horizontal([Constraint::Max(18), Constraint::Fill(1)]).areas(area);

        let [top_right, bottom_right] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)]).areas(right);

        self.render_nodes(left, buf);
        self.render_simulation(top_right, buf);
        self.render_stats(bottom_right, buf);
    }

    fn render_nodes(&mut self, area: Rect, buf: &mut Buffer) {
        let left_block = Block::new()
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .title("Nodes")
            .bg(BG_COLOR)
            .fg(TEXT_COLOR);

        let items = self
            .nodes
            .iter()
            .map(|x| x.repr.as_str())
            .collect::<Vec<&str>>();
        //let items = ["Drone  #12321","Drone  #12321","Drone  #12321","Drone  #12321", "Client #22343", "Server #32342"];
        let list = List::new(items)
            .block(Block::bordered().title("List"))
            .style(Style::new().white())
            .highlight_style(Style::new().bold().bg(HIGHLIGHT_COLOR))
            .highlight_symbol("»")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom)
            .block(left_block)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.node_list_state);
    }

    fn render_simulation(&self, area: Rect, buf: &mut Buffer) {
        let top_right_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            ..symbols::border::PLAIN
        };

        let block = Block::new()
            .border_set(top_right_border_set)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title("Simulation")
            .bg(BG_COLOR)
            .fg(TEXT_COLOR);

        // redo to avoid panic
        let max_x = self.nodes.iter().map(|n| n.x).max().unwrap();
        let max_y = self.nodes.iter().map(|n| n.y).max().unwrap();

        let scale_x = area.width as f64 / max_x as f64;
        let scale_y = area.height as f64 / max_y as f64;

        let canvas_border_offset: f64 = 5.0;

        let canvas = Canvas::default()
            .block(block)
            .marker(Marker::Braille)
            .paint(|ctx| {
                let mut checked: HashSet<&Node> = HashSet::new();

                for (p1, n1) in self.nodes.iter().enumerate() {
                    checked.insert(&n1);
                    for (p2, n2) in self.nodes.iter().enumerate() {
                        if !checked.contains(&n2) {
                            // && rand::random::<bool>()
                            let mut c: Color = Color::DarkGray;
                            if let Some(selected_index) = self.node_list_state.selected() {
                                if (selected_index == p1 || selected_index == p2) {
                                    c = HIGHLIGHT_COLOR;
                                }
                            }

                            let line = ratatui::widgets::canvas::Line {
                                x1: (n1.x as f64) * scale_x + canvas_border_offset,
                                y1: (n1.y as f64) * scale_y + canvas_border_offset,
                                x2: (n2.x as f64) * scale_x + canvas_border_offset,
                                y2: (n2.y as f64) * scale_y + canvas_border_offset,
                                color: c,
                            };
                            ctx.draw(&line);
                        }
                    }
                }

                ctx.layer();

                if let Some(id1) = self.node_list_state.selected() {
                    let n1 = &self.nodes[id1];
                    for n2 in self.nodes.iter() {
                        let line = ratatui::widgets::canvas::Line {
                            x1: (n1.x as f64) * scale_x + canvas_border_offset,
                            y1: (n1.y as f64) * scale_y + canvas_border_offset,
                            x2: (n2.x as f64) * scale_x + canvas_border_offset,
                            y2: (n2.y as f64) * scale_y + canvas_border_offset,
                            color: HIGHLIGHT_COLOR,
                        };
                        ctx.draw(&line);
                    }
                }

                for (pos, n) in self.nodes.iter().enumerate() {
                    let tx = (n.x as f64) * scale_x + canvas_border_offset;
                    let ty = (n.y as f64) * scale_y + canvas_border_offset;

                    let mut s = Style::new().fg(TEXT_COLOR);
                    let mut c: char;
                    let mut bl: char;
                    let mut br: char;
                    match n.kind {
                        NodeKind::Drone => {
                            s = s.bg(DRONE_COLOR);
                            c = 'D';
                            bl = '(';
                            br = ')';
                        }
                        NodeKind::Client => {
                            s = s.bg(CLIENT_COLOR);
                            c = 'C';
                            bl = '[';
                            br = ']';
                        }
                        NodeKind::Server => {
                            s = s.bg(SERVER_COLOR);
                            c = 'S';
                            bl = '[';
                            br = ']';
                        }
                    }

                    if let Some(selected_index) = self.node_list_state.selected() {
                        if (selected_index == pos) {
                            // bl = '»';
                            // br = '«';
                            s = s.bg(HIGHLIGHT_COLOR);
                            s = s.fg(BG_COLOR);
                            s = s.bold();
                        }
                    }

                    ctx.print(
                        tx - 0.5,
                        ty - 0.5,
                        Line::styled(format!("{}{}{}", bl, c, br), s),
                    );
                }
            })
            .background_color(BG_COLOR)
            .x_bounds([0.0, area.width as f64 + (canvas_border_offset * 2.0)])
            .y_bounds([0.0, area.height as f64 + (canvas_border_offset * 2.0)]);

        canvas.render(area, buf);
    }

    fn render_stats(&self, area: Rect, buf: &mut Buffer) {
        let collapsed_top_and_left_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.vertical_right,
            top_right: symbols::line::NORMAL.vertical_left,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..symbols::border::PLAIN
        };

        // let bottom_right_block =
        Block::new()
            .border_set(collapsed_top_and_left_border_set)
            .borders(Borders::ALL)
            .title("Stats")
            .bg(BG_COLOR)
            .fg(TEXT_COLOR)
            .render(area, buf);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let start_keys = [
            ("↑", "Up"),
            ("↓", "Down"),
            ("+", "Open initialization file"),
            ("q", "Quit"),
        ];

        let main_keys = [
            ("↑", "Up"),
            ("↓", "Down"),
            ("e", "Edit node"),
            ("m", "Move node"),
            ("+", "Add node"),
            ("q", "Quit"),
        ];

        let move_keys = [
            ("↑/↓/→/←", "Move"),
            ("Enter", "Ok"), 
            ("q", "Quit")
        ];

        let keys: &[(&str, &str)] = match self.screen {
            Screen::Main => &main_keys,
            Screen::Start => &start_keys,
            Screen::Move => &move_keys,
        };

        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(
                    format!(" {key} "),
                    Style::new().fg(INVERTED_TEXT_COLOR).bg(HIGHLIGHT_COLOR),
                );
                let desc = Span::styled(
                    format!(" {desc} "),
                    Style::new().fg(TEXT_COLOR).bg(BOTTOMPANEL_BG),
                );
                [key, desc]
            })
            .collect();

        Line::from(spans)
            .centered()
            .style((Color::Yellow, BOTTOMPANEL_BG))
            .render(area, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main, footer] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        self.render_footer(footer, buf);

        match self.screen {
            Screen::Start => {
                self.render_start(main, buf);
            }
            Screen::Main => {
                self.render_main(main, buf);
            }
            Screen::Move => {
                self.render_main(main, buf);
            }
        }
    }
}
