mod agent;
mod app;
mod game;
mod minimax_agent;
mod rl_agent;

use std::{
    io::{self, Stdout, stdout},
    time::Duration,
    u64::MAX,
};

use agent::Agents;
use app::render;
use color_eyre::Result;
use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use game::{GameConfigPreset, GameState, Player};
use ratatui::{DefaultTerminal, Terminal, prelude::CrosstermBackend};

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = init()?;
    let app_result = run(&mut terminal);
    if let Err(err) = restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            err
        );
    }
    app_result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunSpeed {
    Slow,
    Fast,
    Instant,
    Manual,
}

impl ToString for RunSpeed {
    fn to_string(&self) -> String {
        match self {
            RunSpeed::Slow => "Slow".to_string(),
            RunSpeed::Fast => "Fast".to_string(),
            RunSpeed::Instant => "Instant".to_string(),
            RunSpeed::Manual => "Manual".to_string(),
        }
    }
}

impl RunSpeed {
    pub fn time(&self) -> Duration {
        match self {
            RunSpeed::Slow => Duration::from_millis(1000),
            RunSpeed::Fast => Duration::from_millis(250),
            RunSpeed::Instant => Duration::from_millis(0),
            RunSpeed::Manual => Duration::from_millis(MAX),
        }
    }
}
/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    set_panic_hook();
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore(); // ignore any errors as we are already failing
        hook(panic_info);
    }));
}

/// Restore the terminal to its original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = app::App::new();
    let mut run_speed = RunSpeed::Manual;

    loop {
        terminal.draw(|frame| render(frame, &mut app, &run_speed))?;

        let event_exists = event::poll(run_speed.time())?;
        if event_exists || run_speed == RunSpeed::Manual {
            let event = event::read()?;
            match event {
                event::Event::Key(key) => match key.code {
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Char('s') => run_speed = RunSpeed::Slow,
                    KeyCode::Char('f') => run_speed = RunSpeed::Fast,
                    KeyCode::Char('i') => run_speed = RunSpeed::Instant,
                    KeyCode::Char('m') => run_speed = RunSpeed::Manual,
                    KeyCode::Char('r') => {
                        app.menu_open = false;
                        app.reset();
                    }
                    KeyCode::Char('p') => {
                        app.menu_open = true;
                    }
                    KeyCode::Char(' ') => app.step(None)?,

                    // List
                    KeyCode::Char('g') => app.agent_list.state.select_first(),
                    KeyCode::Char('G') => app.agent_list.state.select_last(),
                    KeyCode::Char('j') | KeyCode::Down => app.agent_list.state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.agent_list.state.select_previous(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        let selected = app.agent_list.state.selected();
                        // first two are the players
                        match selected {
                            Some(0) => app.agent_list.selected_player = Player::Yellow,
                            Some(1) => app.agent_list.selected_player = Player::Red,
                            Some(x) => {
                                // Handle from AGENTS list
                                let agent_index = x - 2;
                                app.set_agent(
                                    app.agent_list.selected_player,
                                    Agents::agent_types()[agent_index].clone(),
                                );
                            }
                            None => {}
                        }
                        app.agent_list.state.select(None);
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        // Cycle through config
                        if app
                            .config_list
                            .state
                            .selected()
                            .is_none_or(|i| i == GameConfigPreset::amount_of_presets() - 1)
                        {
                            app.config_list.state.select_first();
                        } else {
                            app.config_list.state.select_next();
                        }
                        // Reset game with new config
                        app.config_list.selected_game = GameConfigPreset::from_index(
                            app.config_list.state.selected().unwrap_or(0),
                        );
                        app.reset();
                    }
                    _ => {
                        if *app.game.state() == GameState::InProgress {
                            app.step(Some(event))?;
                        }
                    }
                },
                _ => {}
            }
        } else {
            if *app.game.state() == GameState::InProgress {
                app.step(None)?;
            }
        }
    }
}
