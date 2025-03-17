use color_eyre::eyre;
use crossterm::event::Event;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{
        Block, BorderType, Borders, List, ListState, Padding, Paragraph, StatefulWidget, Wrap,
    },
};

use crate::{
    RunSpeed,
    agent::{Agent, Agents},
    game::{Game, GameConfigPreset, GameState, GridWidget, Player},
};

pub struct App {
    pub game: Game,
    pub yellow_agent: Box<dyn Agent>,
    pub red_agent: Box<dyn Agent>,
    pub yellow_agent_type: Agents,
    pub red_agent_type: Agents,

    pub menu_open: bool,
    pub agent_list: AgentList,
    pub config_list: GameConfigList,
}

impl App {
    pub fn new() -> Self {
        let game = Game::new();
        let yellow_agent_type = Agents::Human;
        let red_agent_type = Agents::Minimax(1);
        let yellow_agent =
            Agents::create_agent(&Agents::agent_names()[0], Player::Yellow, *game.config());
        let red_agent =
            Agents::create_agent(&Agents::agent_names()[3], Player::Red, *game.config());
        App {
            game,
            yellow_agent,
            red_agent,
            yellow_agent_type,
            red_agent_type,
            menu_open: false,
            agent_list: AgentList {
                selected_player: Player::Yellow,
                state: ListState::default().with_selected(Some(0)),
            },
            config_list: GameConfigList {
                selected_game: GameConfigPreset::default(),
                state: ListState::default().with_selected(Some(0)),
            },
        }
    }

    pub fn reset(&mut self) {
        self.game = Game::with_config(self.config_list.selected_game.into_config());
        // Reset agents (may have different config)
        self.yellow_agent = self
            .yellow_agent_type
            .clone()
            .into_agent(Player::Yellow, self.config_list.selected_game.into_config());
        self.red_agent = self
            .red_agent_type
            .clone()
            .into_agent(Player::Red, self.config_list.selected_game.into_config());
    }

    pub fn set_agent(&mut self, player: Player, agent: Agents) {
        match player {
            Player::Yellow => {
                self.yellow_agent_type = agent;
                self.yellow_agent = self
                    .yellow_agent_type
                    .clone()
                    .into_agent(Player::Yellow, self.config_list.selected_game.into_config());
            }
            Player::Red => {
                self.red_agent_type = agent;
                self.red_agent = self
                    .red_agent_type
                    .clone()
                    .into_agent(Player::Red, self.config_list.selected_game.into_config());
            }
        }
    }

    fn current_player_is_human(&self) -> bool {
        match self.game.current_player() {
            crate::game::Player::Yellow => self.yellow_agent.is_human(),
            crate::game::Player::Red => self.red_agent.is_human(),
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

    fn render_agent_list(&mut self, area: Rect, buf: &mut Buffer) {
        // Define selectable options
        let mut options = vec![
            "Select to change Yellow".to_string(),
            "Select to change Red".to_string(),
        ];
        options.append(&mut Agents::agent_names());

        // Render selectable options
        let list = List::new(options)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title_bottom(Line::from(vec![
                        "Select options by moving up ".into(),
                        "<k> or <↑>".blue(),
                        " or down ".into(),
                        "<j> or <↓>".blue(),
                        " Then select using ".into(),
                        "<Enter>".blue(),
                    ]))
                    .title_top(Line::from(
                        format!(
                            " Select Agent for {} ",
                            self.agent_list.selected_player.to_string()
                        )
                        .blue(),
                    )),
            )
            .highlight_style(Style::default().fg(Color::Blue))
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, &mut self.agent_list.state);
    }

    fn render_config_list(&mut self, area: Rect, buf: &mut Buffer) {
        let list = List::new(vec![
            "Standard".to_string(),
            "Small".to_string(),
            "Large".to_string(),
            "Huge".to_string(),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title_bottom(Line::from(vec!["Cycle options with ".into(), "<c>".blue()]))
                .title_top(Line::from(" Select Game Config ".blue())),
        )
        .highlight_style(Style::default().fg(Color::Blue))
        .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, &mut self.config_list.state);
    }
}

pub struct AgentList {
    pub selected_player: Player,
    pub state: ListState,
}

pub struct GameConfigList {
    pub selected_game: GameConfigPreset,
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

    let [left_menu, right_menu] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(70),
            ratatui::layout::Constraint::Percentage(30),
        ])
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
        Line::from(format!(
            "Game Config: {}x{}",
            app.game.config().cols,
            app.game.config().rows
        )),
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
        app.render_agent_list(left_menu, frame.buffer_mut());
        app.render_config_list(right_menu, frame.buffer_mut());
    } else {
        frame.render_widget(grid, horizontal_layout[0]);
    }
}
