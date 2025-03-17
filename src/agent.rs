use crossterm::event::{Event, KeyCode};

use crate::{
    game::{Game, Player},
    minimax_agent::MinimaxAgent,
    rl_agent::RLAgent,
};

/// Agent trait for making moves in a game.
pub trait Agent {
    /// Get an action based on the current game state and optional event (for input).
    fn get_action(&mut self, board: &Game, event: Option<Event>) -> Option<usize>;

    /// Gets the type of the agent.
    fn get_type(&self) -> String;

    /// Check if the agent is human or not.
    fn is_human(&self) -> bool;

    /// Learn from the game state (if learning agent)
    fn learn(&mut self, board: &Game, player: Player);
}

pub struct AgentFactory;

impl AgentFactory {
    pub fn create_agent(agent_type: &str, agent_color: Player) -> Box<dyn Agent> {
        match agent_type {
            "Human" => Box::new(HumanAgent),
            "Random" => Box::new(RandomAgent),
            "Greedy" => Box::new(GreedyAgent),
            "Minimax (1)" => Box::new(MinimaxAgent { max_depth: 1 }),
            "Minimax (3)" => Box::new(MinimaxAgent { max_depth: 3 }),
            "Minimax (5)" => Box::new(MinimaxAgent { max_depth: 5 }),
            "Minimax (7)" => Box::new(MinimaxAgent { max_depth: 7 }),
            "Minimax (9)" => Box::new(MinimaxAgent { max_depth: 9 }),
            "RL (0.2)" => Box::new(RLAgent::new(0.2, false, agent_color)),
            "RL (Learning)" => Box::new(RLAgent::new(0.4, true, agent_color)),
            _ => panic!("Invalid agent type"),
        }
    }

    pub fn agent_types() -> Vec<String> {
        vec![
            "Human".to_string(),
            "Random".to_string(),
            "Greedy".to_string(),
            "Minimax (1)".to_string(),
            "Minimax (3)".to_string(),
            "Minimax (5)".to_string(),
            "Minimax (7)".to_string(),
            "Minimax (9)".to_string(),
            "RL (0.2)".to_string(),
            "RL (Learning)".to_string(),
        ]
    }
}

/// Human agent that makes moves based on user input.
pub struct HumanAgent;

impl Agent for HumanAgent {
    fn get_action(&mut self, _board: &Game, event: Option<Event>) -> Option<usize> {
        // We will try to get valid column
        match event {
            Some(Event::Key(key)) => match key.code {
                KeyCode::Char('1') => return Some(0),
                KeyCode::Char('2') => return Some(1),
                KeyCode::Char('3') => return Some(2),
                KeyCode::Char('4') => return Some(3),
                KeyCode::Char('5') => return Some(4),
                KeyCode::Char('6') => return Some(5),
                KeyCode::Char('7') => return Some(6),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_type(&self) -> String {
        "Human".to_string()
    }

    fn is_human(&self) -> bool {
        true
    }

    fn learn(&mut self, _board: &Game, _player: Player) {
        // No learning for human agent
    }
}

/// Ai agent which makes a random move
pub struct RandomAgent;

impl RandomAgent {}

impl Agent for RandomAgent {
    fn get_action(&mut self, board: &Game, _event: Option<Event>) -> Option<usize> {
        use rand::Rng;
        let mut rng = rand::rng();

        // Get the valid moves (empty columns)
        let valid_moves: Vec<usize> = (0..7).filter(|&col| !board.is_column_full(col)).collect();

        if valid_moves.is_empty() {
            return None;
        }

        // Select a random valid move
        let random_index = rng.random_range(0..valid_moves.len());
        Some(valid_moves[random_index])
    }

    fn get_type(&self) -> String {
        "Random".to_string()
    }

    fn is_human(&self) -> bool {
        false
    }

    fn learn(&mut self, _board: &Game, _player: Player) {
        // No learning for random agent
    }
}

/// A simple greedy agent which chooses columns with adjacent tiles of the same color
pub struct GreedyAgent;

impl GreedyAgent {
    /// Count adjacent tiles of the same color after placing in a column
    fn evaluate_move(&self, board: &Game, col: usize) -> i32 {
        // Clone board and make move
        let mut board_copy = board.clone();
        if board_copy.place(col).is_none() {
            return -1; // Invalid move
        }

        let player = board.current_player();
        let mut score = 0;

        // Check entire board for clusters
        for row in 0..6 {
            for col in 0..7 {
                if let Some(piece) = board_copy.get_cell(row, col) {
                    if piece == player {
                        // Add points for each neighbor of same color
                        // Check 8 directions: horizontal, vertical, and two diagonals
                        let directions = [
                            (-1, -1),
                            (-1, 0),
                            (-1, 1),
                            (0, -1),
                            (0, 1),
                            (1, -1),
                            (1, 0),
                            (1, 1),
                        ];

                        for (dr, dc) in directions.iter() {
                            let new_row = row as i32 + dr;
                            let new_col = col as i32 + dc;

                            // Check bounds
                            if new_row >= 0 && new_row < 6 && new_col >= 0 && new_col < 7 {
                                if let Some(neighbor) =
                                    board_copy.get_cell(new_row as usize, new_col as usize)
                                {
                                    if neighbor == player {
                                        score += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        score
    }
}

impl Agent for GreedyAgent {
    fn get_action(&mut self, board: &Game, _event: Option<Event>) -> Option<usize> {
        // Get valid moves
        let valid_moves: Vec<usize> = (0..7).filter(|&col| !board.is_column_full(col)).collect();

        if valid_moves.is_empty() {
            return None;
        }

        // Find move with highest score
        let mut best_score = -1;
        let mut best_moves = Vec::new();

        for &col in &valid_moves {
            let score = self.evaluate_move(board, col);

            if score > best_score {
                best_score = score;
                best_moves.clear();
                best_moves.push(col);
            } else if score == best_score {
                best_moves.push(col);
            }
        }

        // If we have multiple best moves, prefer center columns
        if best_moves.len() > 1 {
            // Sort by distance from center (column 3)
            best_moves.sort_by_key(|&col| (col as i32 - 3).abs());
        }

        Some(best_moves[0])
    }

    fn get_type(&self) -> String {
        "Greedy".to_string()
    }

    fn is_human(&self) -> bool {
        false
    }

    fn learn(&mut self, _board: &Game, _player: Player) {
        // No learning for greedy agent
    }
}
