use std::{collections::HashMap, fs, io, path::PathBuf};

use crossterm::event::Event;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    agent::Agent,
    game::{Game, GameConfig, GameState, Player},
};

/// RL agent implementation using Q-learning algorithm with history
#[derive(Serialize, Deserialize)]
pub struct RLAgent {
    // Q-table mapping board state to action values
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    q_table: HashMap<String, Vec<f64>>,

    #[serde(skip)]
    epsilon: f64,
    #[serde(skip)]
    learning: bool,
    #[serde(skip)]
    turn: usize,
    #[serde(skip)]
    agent_color: Player,
    // Game history for learning from sequences
    #[serde(skip)]
    move_history: Vec<(String, usize)>,

    #[serde(skip)]
    board_config: GameConfig,
}

impl RLAgent {
    const LEARNING_RATE: f64 = 0.15;
    const WIN_REWARD: f64 = 5.0;
    const LOSS_REWARD: f64 = -10.0; // Doubled loss penalty
    const DRAW_REWARD: f64 = 1.0;
    const DURATION_REWARD: f64 = 0.02;
    const MAX_HISTORY: usize = 3; // Number of previous moves to consider

    pub fn new(
        epsilon: f64,
        learning: bool,
        agent_color: Player,
        board_config: GameConfig,
    ) -> Self {
        // Create a new agent
        let mut agent = RLAgent {
            q_table: HashMap::new(),
            epsilon,
            learning,
            agent_color,
            turn: 0,
            move_history: Vec::new(),
            board_config,
        };

        // Try to load existing Q-table if available
        if Self::save_path(&board_config).exists() {
            if let Err(e) = agent.load_q_table() {
                eprintln!("Failed to load Q-table: {}", e);
            }
        }

        agent
    }

    // Computes save path in directory based on game config
    fn save_path(config: &GameConfig) -> PathBuf {
        [
            "connect4",
            "rl_data",
            &format!("q_table_{}x{}.json", config.cols, config.rows),
        ]
        .iter()
        .collect()
    }

    // Convert board to a string representation for the Q-table
    fn board_to_state(&self, board: &Game) -> String {
        let mut state = String::with_capacity(21);

        // For each column, encode the pieces from bottom to top
        for col in 0..board.config().cols {
            let mut col_pieces = Vec::new();

            // Find pieces in this column (from bottom up)
            for row in (0..board.config().rows).rev() {
                if let Some(player) = board.get_cell(row, col) {
                    // agent-centric encoding
                    if player == self.agent_color {
                        col_pieces.push('m');
                    } else {
                        col_pieces.push('o');
                    }
                }
            }

            // Add column encoding: <length><pieces>
            state.push_str(&format!(
                "{}{}",
                col_pieces.len(),
                col_pieces.iter().collect::<String>()
            ));
        }

        state
    }

    // Check if a move would result in an immediate win
    fn is_winning_move(&self, board: &Game, col: usize) -> bool {
        let mut board_copy = board.clone();
        if board_copy.place(col).is_some() {
            match board_copy.state() {
                GameState::Won(_) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    // Select the best action based on Q-values
    fn select_action(&mut self, board: &Game) -> Option<usize> {
        // Get valid moves
        let valid_moves: Vec<usize> = board.valid_moves();

        if valid_moves.is_empty() {
            return None;
        }

        // First priority: Check for winning moves
        for &col in &valid_moves {
            if self.is_winning_move(board, col) {
                return Some(col);
            }
        }

        // With probability epsilon, choose random action (exploration)
        if rand::rng().random::<f64>() < self.epsilon {
            let idx = rand::rng().random_range(0..valid_moves.len());
            return Some(valid_moves[idx]);
        }

        // Otherwise, choose best action (exploitation)
        let state = self.board_to_state(board);
        let zeroes = vec![0.0; board.config().cols];
        let q_values = self.q_table.get(&state).unwrap_or(&zeroes);

        // Find move with highest Q-value
        // If tie, prefer center columns
        let mut best_value = f64::NEG_INFINITY;
        let mut best_moves = Vec::new();

        for &col in &valid_moves {
            let value = q_values.get(col).unwrap_or(&0.0);

            if *value > best_value {
                best_value = *value;
                best_moves.clear();
                best_moves.push(col);
            } else if (*value - best_value).abs() < 0.001 {
                // Tie within small epsilon
                best_moves.push(col);
            }
        }

        // If multiple best moves, prefer center columns
        if best_moves.len() > 1 {
            best_moves.sort_by_key(|&col| (col as i32 - self.board_config.cols as i32 / 2).abs());
        }

        Some(best_moves[0])
    }

    // Update Q-values based on reward
    fn update_q_value(&mut self, state: &str, action: usize, reward: f64) {
        let q_values = self
            .q_table
            .entry(state.to_string())
            .or_insert_with(|| vec![0.0; self.board_config.cols]);

        if q_values.len() <= action {
            q_values.resize(self.board_config.cols, 0.0);
        }

        let old_value = q_values[action];

        // Q-learning update rule
        q_values[action] = old_value + Self::LEARNING_RATE * (reward - old_value);
    }

    // Save Q-table to disk
    fn save_q_table(&self) -> io::Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = Self::save_path(&self.board_config).parent() {
            fs::create_dir_all(parent)?;
        }

        // Only save if we have data
        if self.q_table.is_empty() {
            return Ok(());
        }

        // Serialize and save
        let serialized = serde_json::to_string(&self)?;
        fs::write(Self::save_path(&self.board_config), serialized)?;

        Ok(())
    }

    // Load Q-table from disk
    fn load_q_table(&mut self) -> io::Result<()> {
        let data = fs::read_to_string(Self::save_path(&self.board_config))?;
        let loaded: RLAgent = serde_json::from_str(&data)?;

        self.q_table = loaded.q_table;

        Ok(())
    }
}

impl Agent for RLAgent {
    fn get_action(&mut self, board: &Game, _event: Option<Event>) -> Option<usize> {
        // Increment turn counter
        self.turn += 1;

        let action = self.select_action(board);

        // Record state-action pair for learning
        if let (Some(action), true) = (action, self.learning) {
            let state = self.board_to_state(board);
            self.move_history.push((state, action));

            // Limit history size
            if self.move_history.len() > Self::MAX_HISTORY {
                self.move_history.remove(0);
            }
        }

        action
    }

    fn get_type(&self) -> String {
        if self.learning {
            format!("RL (ε={:.1}, Learning)", self.epsilon)
        } else {
            format!("RL (ε={:.1})", self.epsilon)
        }
    }

    fn is_human(&self) -> bool {
        false
    }

    fn learn(&mut self, board: &Game, player: Player) {
        // Only learn if we're in learning mode and have moves in history
        if !self.learning || self.move_history.is_empty() {
            return;
        }

        // Calculate final reward based on game outcome
        let mut reward = match board.state() {
            GameState::Won(winner) if *winner == player => Self::WIN_REWARD,
            GameState::Won(_) => Self::LOSS_REWARD, // Double penalty for losses
            GameState::Draw => Self::DRAW_REWARD,
            GameState::InProgress => return, // Game not over
        };

        // Apply duration bonus
        let duration_bonus = self.turn as f64 * Self::DURATION_REWARD;

        if reward < 0.0 {
            // for losses, reduce penalty based on game length
            reward += duration_bonus;
        } else {
            // for wins, increase reward by a bit
            reward += duration_bonus * 0.5;
        }

        // Learn from the game history, back propagation from winning state
        let history_len = self.move_history.len();
        for (i, (state, action)) in self.move_history.clone().iter().enumerate().rev() {
            // Scale reward based on position in history
            let position_factor = (i + 1) as f64 / history_len as f64;
            let move_reward = reward * position_factor;

            // For losses, make sure mistakes are still penalized
            let adjusted_reward = if reward < 0.0 && move_reward > -0.5 {
                -0.5 // Minimum penalty for loss-leading moves
            } else {
                move_reward
            };

            // Update Q-value for this state-action pair
            self.update_q_value(state, *action, adjusted_reward);
        }

        // Save updated Q-table
        if let Err(e) = self.save_q_table() {
            eprintln!(
                "Error saving Q-table at {:?}: {}",
                Self::save_path(&self.board_config),
                e
            );
        }

        // Clear history and reset turn counter
        self.move_history.clear();
        self.turn = 0;
    }
}
