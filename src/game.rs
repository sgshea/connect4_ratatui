use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use serde::{Deserialize, Serialize};

// Define player types
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum Player {
    #[default]
    Red,
    Yellow,
}

impl ToString for Player {
    fn to_string(&self) -> String {
        match self {
            Player::Red => "Red".to_string(),
            Player::Yellow => "Yellow".to_string(),
        }
    }
}

// Define game state
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
    InProgress,
    Won(Player),
    Draw,
}

// Configuration for the Connect 4 game
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GameConfig {
    pub rows: usize,
    pub cols: usize,
    pub connect_length: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            rows: 6,
            cols: 7,
            connect_length: 4,
        }
    }
}

// Presets for game config
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum GameConfigPreset {
    #[default]
    Standard,
    Small,
    Large,
    Huge,
}

impl GameConfigPreset {
    pub fn amount_of_presets() -> usize {
        4
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => GameConfigPreset::Standard,
            1 => GameConfigPreset::Small,
            2 => GameConfigPreset::Large,
            3 => GameConfigPreset::Huge,
            _ => GameConfigPreset::Standard,
        }
    }

    pub fn into_config(self) -> GameConfig {
        match self {
            GameConfigPreset::Standard => GameConfig::default(),
            GameConfigPreset::Small => GameConfig {
                rows: 4,
                cols: 4,
                connect_length: 3,
            },
            GameConfigPreset::Large => GameConfig {
                rows: 8,
                cols: 8,
                connect_length: 5,
            },
            GameConfigPreset::Huge => GameConfig {
                rows: 10,
                cols: 10,
                connect_length: 6,
            },
        }
    }
}

// Connect 4 game struct
#[derive(Clone, PartialEq, Debug)]
pub struct Game {
    board: Vec<Vec<Option<Player>>>,
    current_player: Player,
    state: GameState,
    config: GameConfig,
}

impl Game {
    // Create a new game
    pub fn new() -> Self {
        Self::with_config(GameConfig::default())
    }

    pub fn with_config(config: GameConfig) -> Self {
        let board = vec![vec![None; config.cols]; config.rows];
        Game {
            board,
            current_player: Player::Yellow, // Yellow goes first
            state: GameState::InProgress,
            config,
        }
    }

    // Place a piece in the selected column
    pub fn place(&mut self, column: usize) -> Option<GameState> {
        // Check if the game is still in progress
        if self.state != GameState::InProgress {
            return Some(self.state);
        }

        if column >= self.config.cols {
            return None;
        }

        // Find the first empty row in the column (from bottom to top)
        let row = (0..self.config.rows)
            .rev()
            .find(|&row| self.board[row][column].is_none());

        match row {
            Some(row) => {
                // Place the piece
                self.board[row][column] = Some(self.current_player);

                // Change state
                // Check if this move results in a win
                if self.check_win(row, column) {
                    self.state = GameState::Won(self.current_player);
                } else if self.is_board_full() {
                    self.state = GameState::Draw;
                }

                if self.state == GameState::InProgress {
                    // Switch players
                    self.current_player = match self.current_player {
                        Player::Red => Player::Yellow,
                        Player::Yellow => Player::Red,
                    };
                }

                return Some(self.state);
            }
            None => None,
        }
    }

    // Get the current player
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    // Get the current game state
    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn config(&self) -> &GameConfig {
        &self.config
    }

    pub fn valid_moves(&self) -> Vec<usize> {
        let valid_moves: Vec<usize> = (0..self.config.cols)
            .filter(|&col| !self.is_column_full(col))
            .collect();
        valid_moves
    }

    // Check if the move at (row, col) results in a win
    fn check_win(&self, row: usize, col: usize) -> bool {
        // Check horizontal
        if self.count_consecutive(row, col, 0, 1) >= 4 {
            return true;
        }

        // Check vertical
        if self.count_consecutive(row, col, 1, 0) >= 4 {
            return true;
        }

        // Check diagonal (/)
        if self.count_consecutive(row, col, -1, 1) >= 4 {
            return true;
        }

        // Check diagonal (\)
        if self.count_consecutive(row, col, 1, 1) >= 4 {
            return true;
        }

        false
    }
    // Get the winning combination if one exists
    pub fn get_winning_combination(&self) -> Option<Vec<(usize, usize)>> {
        if let GameState::Won(player) = self.state {
            // Check all possible positions for a starting point of a winning combination
            for row in 0..self.config.rows {
                for col in 0..self.config.cols {
                    if self.board[row][col] == Some(player) {
                        // Check horizontal
                        if col + 3 < self.config.cols {
                            let mut valid = true;
                            for i in 1..4 {
                                if self.board[row][col + i] != Some(player) {
                                    valid = false;
                                    break;
                                }
                            }
                            if valid {
                                return Some(vec![
                                    (row, col),
                                    (row, col + 1),
                                    (row, col + 2),
                                    (row, col + 3),
                                ]);
                            }
                        }

                        // Check vertical
                        if row + 3 < self.config.rows {
                            let mut valid = true;
                            for i in 1..4 {
                                if self.board[row + i][col] != Some(player) {
                                    valid = false;
                                    break;
                                }
                            }
                            if valid {
                                return Some(vec![
                                    (row, col),
                                    (row + 1, col),
                                    (row + 2, col),
                                    (row + 3, col),
                                ]);
                            }
                        }

                        // Check diagonal (/)
                        if row >= 3 && col + 3 < self.config.cols {
                            let mut valid = true;
                            for i in 1..4 {
                                if self.board[row - i][col + i] != Some(player) {
                                    valid = false;
                                    break;
                                }
                            }
                            if valid {
                                return Some(vec![
                                    (row, col),
                                    (row - 1, col + 1),
                                    (row - 2, col + 2),
                                    (row - 3, col + 3),
                                ]);
                            }
                        }

                        // Check diagonal (\)
                        if row + 3 < self.config.rows && col + 3 < self.config.cols {
                            let mut valid = true;
                            for i in 1..4 {
                                if self.board[row + i][col + i] != Some(player) {
                                    valid = false;
                                    break;
                                }
                            }
                            if valid {
                                return Some(vec![
                                    (row, col),
                                    (row + 1, col + 1),
                                    (row + 2, col + 2),
                                    (row + 3, col + 3),
                                ]);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    // Count consecutive pieces of the same color in a given direction
    fn count_consecutive(&self, row: usize, col: usize, row_dir: i32, col_dir: i32) -> usize {
        let player = self.board[row][col].unwrap();
        let mut count = 1; // Start with 1 for the piece just placed

        // Count in the positive direction
        count += self.count_direction(row, col, row_dir, col_dir, player);

        // Count in the negative direction
        count += self.count_direction(row, col, -row_dir, -col_dir, player);

        count
    }

    // Helper to count in a specific direction
    fn count_direction(
        &self,
        row: usize,
        col: usize,
        row_dir: i32,
        col_dir: i32,
        player: Player,
    ) -> usize {
        let mut count = 0;
        let mut r = row as i32 + row_dir;
        let mut c = col as i32 + col_dir;

        while r >= 0
            && r < self.config.rows as i32
            && c >= 0
            && c < self.config.cols as i32
            && self.board[r as usize][c as usize] == Some(player)
        {
            count += 1;
            r += row_dir;
            c += col_dir;
        }

        count
    }

    // Check if the board is full (draw condition)
    fn is_board_full(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_some()))
    }

    pub fn is_column_full(&self, col: usize) -> bool {
        self.board.iter().all(|row| row[col].is_some())
    }

    // Get a cell's content
    pub fn get_cell(&self, row: usize, col: usize) -> Option<Player> {
        if row < self.config.rows && col < self.config.cols {
            self.board[row][col]
        } else {
            None
        }
    }
}

pub struct GridWidget<'a> {
    pub game: &'a Game,
}

impl<'a> Widget for GridWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().border_set(border::THICK);

        // Build the grid display

        let mut grid = Text::default();

        // Add column numbers
        let mut header = Line::default();
        for i in 0..self.game.config.cols {
            header.spans.push(format!(" {}  ", i + 1).bold().blue());
        }
        grid.lines.push(header);

        let winner = match self.game.state() {
            GameState::InProgress => None,
            GameState::Won(player) => Some(player),
            GameState::Draw => None,
        };

        let winning_cells = if winner.is_some() {
            self.game.get_winning_combination()
        } else {
            None
        };

        // Add the game board
        for row in 0..self.game.config.rows {
            let mut line = Line::default();
            line.spans.push("│".into()); // Left border

            for col in 0..self.game.config.cols {
                let mut cell = match self.game.get_cell(row, col) {
                    Some(Player::Red) => " ● ".red(),
                    Some(Player::Yellow) => " ● ".yellow(),
                    None => " · ".gray(),
                };
                if let Some(winning_cells) = &winning_cells {
                    if winning_cells.contains(&(row, col)) {
                        cell = cell.on_light_green();
                    }
                }
                line.spans.push(cell);
                line.spans.push("│".into()); // Cell divider
            }

            grid.lines.push(line);

            // Add row separator except after the last row
            if row < self.game.config.rows - 1 {
                let mut separator = Line::default();
                separator.spans.push("├".into());
                for col in 0..self.game.config.cols {
                    separator.spans.push("───".into());
                    if col < self.game.config.cols - 1 {
                        separator.spans.push("┼".into());
                    } else {
                        separator.spans.push("┤".into());
                    }
                }
                grid.lines.push(separator);
            }
        }

        // Add bottom border
        let mut bottom = Line::default();
        bottom.spans.push("└".into());
        for col in 0..self.game.config.cols {
            bottom.spans.push("───".into());
            if col < self.game.config.cols - 1 {
                bottom.spans.push("┴".into());
            } else {
                bottom.spans.push("┘".into());
            }
        }
        grid.lines.push(bottom);

        Paragraph::new(grid)
            .centered()
            .block(block)
            .render(area, buf)
    }
}
