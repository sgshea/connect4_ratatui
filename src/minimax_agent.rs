use crossterm::event::Event;

use crate::{
    agent::Agent,
    game::{Game, GameState, Player},
};

/// AI agent using minimax algorithm with alpha-beta pruning
pub struct MinimaxAgent {
    pub max_depth: usize,
}

impl MinimaxAgent {
    pub fn new(max_depth: usize) -> Self {
        MinimaxAgent { max_depth }
    }

    /// Minimax algorithm with alpha-beta pruning
    fn minimax(
        &self,
        player: Player,
        board: &Game,
        depth: usize,
        alpha: i32,
        beta: i32,
        is_maximizing: bool,
    ) -> i32 {
        // Evaluate the current board state
        let board_state = self.evaluate_board(board);

        // Terminal conditions
        match board_state {
            GameState::Won(p) => {
                return if p == player { 1000 } else { -1000 };
            }
            GameState::Draw => return 0,
            GameState::InProgress => {
                // If we've reached max depth, evaluate the position
                if depth == 0 {
                    return self.eval_position(board);
                }
            }
        }

        // Get valid actions based on the board state
        let valid_moves: Vec<usize> = (0..7).filter(|&col| !board.is_column_full(col)).collect();

        if valid_moves.is_empty() {
            return 0; // No valid moves, treat as neutral
        }

        if is_maximizing {
            let mut max_eval = i32::MIN;
            let mut alpha = alpha;

            for &col in &valid_moves {
                let mut board_copy = board.clone();
                if board_copy.place(col).is_some() {
                    let eval = self.minimax(player, &board_copy, depth - 1, alpha, beta, false);
                    max_eval = max_eval.max(eval);
                    alpha = alpha.max(eval);
                    if beta <= alpha {
                        break; // Beta cutoff
                    }
                }
            }

            max_eval
        } else {
            let mut min_eval = i32::MAX;
            let mut beta = beta;

            for &col in &valid_moves {
                let mut board_copy = board.clone();
                if board_copy.place(col).is_some() {
                    let eval = self.minimax(player, &board_copy, depth - 1, alpha, beta, true);
                    min_eval = min_eval.min(eval);
                    beta = beta.min(eval);
                    if beta <= alpha {
                        break; // Alpha cutoff
                    }
                }
            }

            min_eval
        }
    }

    /// Evaluate if the board is in a terminal state
    fn evaluate_board(&self, board: &Game) -> GameState {
        // The game already tracks its state, so we can just return it
        board.state().clone()
    }

    /// Checks if playing in the given column would result in a win
    fn is_winning_move(&self, board: &Game, column: usize, player: Player) -> bool {
        let mut board_copy = board.clone();

        // Try to place a piece for the specified player
        let current_player = board_copy.current_player();
        if current_player != player {
            // If it's not the player's turn, we need two moves to test
            // First, place a piece for the current player in a different column if possible
            for col in 0..7 {
                if col != column && !board_copy.is_column_full(col) {
                    if board_copy.place(col).is_some() {
                        break;
                    }
                }
            }

            // Now check if the second player (our target) can make a winning move
            if board_copy.current_player() != player {
                return false; // Couldn't set up the test properly
            }
        }

        // Place the piece and check if it results in a win
        if board_copy.place(column).is_some() {
            match board_copy.state() {
                GameState::Won(p) if *p == player => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Evaluation function for non-terminal board positions
    fn eval_position(&self, board: &Game) -> i32 {
        let mut score = 0;
        let my_color = board.current_player();
        let opponent_color = match my_color {
            Player::Yellow => Player::Red,
            Player::Red => Player::Yellow,
        };

        // Evaluate center control (column 3, which is index 3)
        let center_col = 3;
        for row in 0..6 {
            match board.get_cell(row, center_col) {
                Some(player) if player == my_color => score += 5, // Prioritize center control
                Some(player) if player == opponent_color => score -= 2, // Penalize opponent's center control
                _ => {}
            }
        }

        // Evaluate pieces with their positions
        for row in 0..6 {
            for col in 0..7 {
                match board.get_cell(row, col) {
                    Some(player) if player == my_color => {
                        // Pieces closer to the center are more valuable
                        score += 5 - (col as i32 - center_col as i32).abs();

                        // Check for adjacent friendly pieces
                        if self.has_adjacent_same_color(board, row, col, my_color) {
                            score += 2;
                        }
                    }
                    Some(player) if player == opponent_color => {
                        // Opponent pieces are bad (especially in the center)
                        score -= 6 - (col as i32 - center_col as i32).abs();

                        // Check for adjacent enemy pieces (potential threats)
                        if self.has_adjacent_same_color(board, row, col, opponent_color) {
                            score -= 2;
                        }
                    }
                    _ => {}
                }
            }
        }

        score
    }

    /// Helper method to check if a position has adjacent pieces of the same color
    fn has_adjacent_same_color(&self, board: &Game, row: usize, col: usize, color: Player) -> bool {
        let directions = [
            (0, -1), // left
            (0, 1),  // right
            (1, 0),  // down
            (1, -1), // diagonal down-left
            (1, 1),  // diagonal down-right
        ];

        for &(row_dir, col_dir) in &directions {
            let new_row = row as i32 + row_dir;
            let new_col = col as i32 + col_dir;

            // Check if position is valid and has the same color
            if new_row >= 0 && new_row < 6 && new_col >= 0 && new_col < 7 {
                if let Some(player) = board.get_cell(new_row as usize, new_col as usize) {
                    if player == color {
                        return true;
                    }
                }
            }
        }

        false
    }
}

impl Agent for MinimaxAgent {
    fn get_action(&mut self, board: &Game, _event: Option<Event>) -> Option<usize> {
        let valid_moves: Vec<usize> = (0..7).filter(|&col| !board.is_column_full(col)).collect();

        // If only one action is available, return it immediately
        if valid_moves.len() == 1 {
            return Some(valid_moves[0]);
        }

        // This is us
        let current_player = board.current_player();

        // Check if we can win in one move
        for &col in &valid_moves {
            if self.is_winning_move(board, col, current_player) {
                return Some(col);
            }
        }

        // Check if we need to block opponent's winning move
        let opponent = match current_player {
            Player::Yellow => Player::Red,
            Player::Red => Player::Yellow,
        };

        for &col in &valid_moves {
            if self.is_winning_move(board, col, opponent) {
                return Some(col);
            }
        }

        // Run minimax to find the best move
        let mut best_col = valid_moves[valid_moves.len() - 1];
        let mut best_value = i32::MIN;
        let mut alpha = i32::MIN;
        let beta = i32::MAX;

        for &col in &valid_moves {
            let mut board_copy = board.clone();
            if board_copy.place(col).is_some() {
                let value = self.minimax(
                    current_player,
                    &board_copy,
                    self.max_depth - 1,
                    alpha,
                    beta,
                    false,
                );

                if value > best_value {
                    best_value = value;
                    best_col = col;
                }
                alpha = alpha.max(best_value);
            }
        }

        Some(best_col)
    }

    fn get_type(&self) -> String {
        // Display type + depth
        format!("Minimax ({})", self.max_depth)
    }

    fn is_human(&self) -> bool {
        false
    }

    fn learn(&mut self, _board: &Game, _player: Player) {
        // No learning for minimax agent
    }
}
