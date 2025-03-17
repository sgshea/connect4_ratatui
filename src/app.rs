use color_eyre::eyre;
use crossterm::event::Event;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{
        Block, BorderType, Borders, List, ListState, Padding, Paragraph, StatefulWidget, Wrap,
    },
};

use crate::{
    RunSpeed,
    agent::{Agent, AgentFactory},
    game::{Game, GameState, GridWidget, Player},
};

pub struct App {
    pub game: Game,
    pub yellow_agent: Box<dyn Agent>,
    pub red_agent: Box<dyn Agent>,

    // Which ones are human agents
    pub human_agents: (bool, bool),

    pub menu_open: bool,
    pub list: AgentList,
}

impl App {
    pub fn new(yellow_agent: Box<dyn Agent>, red_agent: Box<dyn Agent>) -> Self {
        App {
            game: Game::new(),
            human_agents: (yellow_agent.is_human(), red_agent.is_human()),
            yellow_agent,
            red_agent,
            menu_open: false,
            list: AgentList {
                selected_player: Player::Yellow,
                state: ListState::default().with_selected(Some(0)),
            },
        }
    }

    pub fn reset(&mut self) {
        self.game = Game::new();
    }

    pub fn set_agent(&mut self, player: Player, agent: Box<dyn Agent>) {
        match player {
            Player::Yellow => self.yellow_agent = agent,
            Player::Red => self.red_agent = agent,
        }

        // Set human_agents
        self.human_agents = (self.yellow_agent.is_human(), self.red_agent.is_human());
    }

    fn current_player_is_human(&self) -> bool {
        match self.game.current_player() {
            crate::game::Player::Yellow => self.human_agents.0,
            crate::game::Player::Red => self.human_agents.1,
        }
    }

    pub fn step(&mut self, event: Option<Event>) -> eyre::Result<()> {
        let event = if self.current_player_is_human() {
            event
        } else {
            None
        };

        match self.game.current_player() {
            crate::game::Player::Yellow => {
                let action = self.yellow_agent.get_action(&self.game, event);
                if let Some(action) = action {
                    let state = self.game.place(action);
                    if state.is_some_and(|s| s != GameState::InProgress) {
                        // Handle learning
                        self.yellow_agent.learn(&self.game, Player::Yellow);
                    }
                }
            }
            crate::game::Player::Red => {
                let action = self.red_agent.get_action(&self.game, event);
                if let Some(action) = action {
                    let state = self.game.place(action);
                    if state.is_some_and(|s| s != GameState::InProgress) {
                        // Handle learning
                        self.red_agent.learn(&self.game, Player::Red);
                    }
                }
            }
        }
        Ok(())
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        // Define selectable options
        let mut options = vec![
            "Select to change Yellow".to_string(),
            "Select to change Red".to_string(),
        ];
        options.append(&mut AgentFactory::agent_types());

        // Render selectable options
        let list = List::new(options)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title_top(Line::from(
                        format!(
                            " Select Agent for {} ",
                            self.list.selected_player.to_string()
                        )
                        .blue(),
                    )),
            )
            .highlight_style(Style::default().fg(Color::Blue))
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, &mut self.list.state);
    }
}

pub struct AgentList {
    pub selected_player: Player,
    pub state: ListState,
}

pub fn render(frame: &mut Frame, app: &mut App, current_speed: &RunSpeed) {
    let grid = GridWidget { game: &app.game };

    let area = frame.area();

    let global_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_top(Line::from(" Connect 4 ".bold()).red())
        .padding(Padding::horizontal(1));

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(70),
            ratatui::layout::Constraint::Percentage(30),
        ])
        .flex(Flex::Center)
        .split(global_block.inner(area));

    let [grid_area] = Layout::vertical([Constraint::Length(25)])
        .flex(Flex::Center)
        .areas(horizontal_layout[0]);

    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title_top(Line::from(" Game Info ".bold()).green())
        .padding(Padding::horizontal(1));

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(60),
        ])
        .split(right_block.inner(horizontal_layout[1]));

    let status = match app.game.state() {
        GameState::InProgress => {
            // Match color of player
            match app.game.current_player() {
                Player::Red => {
                    Line::from(format!("Current player: {:?}", app.game.current_player()).red())
                }
                Player::Yellow => {
                    Line::from(format!("Current player: {:?}", app.game.current_player()).yellow())
                }
            }
        }
        GameState::Won(player) => Line::from(format!("Player {:?} wins!", player).green()),
        GameState::Draw => Line::from("Game ended in a draw".yellow()),
    };

    let player_info = Paragraph::new(vec![
        status,
        Line::from(" "),
        Line::from(format!("Player 1 [{}]", app.yellow_agent.get_type()).yellow()),
        Line::from(format!("Player 2 [{}]", app.red_agent.get_type()).red()),
    ]);

    let mut instructions = vec![
        Line::from(" "),
        Line::from(vec![
            "Quit ".into(),
            "<q>".red(),
            " Reset Play ".into(),
            "<r>".blue(),
            " Menu ".into(),
            "<p>".blue(),
        ]),
        Line::from(" "),
        Line::from(vec![
            "Current speed: ".into(),
            current_speed.to_string().into(),
        ]),
        Line::from(vec![
            "Slow ".into(),
            "<s> ".blue(),
            "Fast ".into(),
            "<f> ".blue(),
            "Instant ".into(),
            "<i> ".blue(),
            "Manual (Press Space to increment turn) ".into(),
            "<m> ".blue(),
        ]),
    ];

    // Add extra instruction if any human player
    if app.yellow_agent.is_human() || app.red_agent.is_human() {
        instructions.push(Line::from(
            "Drop a piece by entering number of column.".green(),
        ));
    }

    frame.render_widget(global_block, area);
    frame.render_widget(right_block, horizontal_layout[1]);
    frame.render_widget(
        Paragraph::new(instructions).wrap(Wrap { trim: true }),
        vertical_layout[1],
    );
    frame.render_widget(player_info, vertical_layout[0]);

    if app.menu_open {
        app.render_list(grid_area, frame.buffer_mut());
    } else {
        frame.render_widget(grid, grid_area);
    }
}
