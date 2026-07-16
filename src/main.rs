use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{
        self,
        Color::{self, Cyan, Yellow},
        Stylize,
    },
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use rand::seq::SliceRandom;

const GAME_BOARD_W: usize = 10;
const GAME_BOARD_H: usize = 20;
const NEXT_BOARD_W: usize = 6;
const NEXT_BOARD_H: usize = 6;
const GAME_BOARD_WIN_W: usize = GAME_BOARD_W * 3;
const GAME_BOARD_WIN_H: usize = GAME_BOARD_H;
const NEXT_BOARD_WIN_W: usize = NEXT_BOARD_W * 3;
const NEXT_BOARD_WIN_H: usize = NEXT_BOARD_H;

#[derive(Default, Clone)]
enum TetrominoType {
    #[default]
    Square,
    Line,
    Squiggly,
    ReverseSquiggly,
    TBlock,
    LBlock,
    ReverseLBlock,
}

#[derive(Default)]
struct GameState {
    active_piece: Tetromino,
    piece_x: i32,
    piece_y: i32,
    game_board: [[bool; GAME_BOARD_W]; GAME_BOARD_H],

    bag: Vec<TetrominoType>,

    game_win_buf: [[u8; GAME_BOARD_W]; GAME_BOARD_H],
    next_win_buf: [[u8; NEXT_BOARD_W]; NEXT_BOARD_H],
    is_running: bool,
}

impl GameState {
    fn new() -> Self {
        Self {
            is_running: true,
            ..Default::default()
        }
    }

    fn new_piece(&mut self) -> Tetromino {
        if self.bag.is_empty() {
            self.bag = vec![
                TetrominoType::Square,
                TetrominoType::Line,
                TetrominoType::Squiggly,
                TetrominoType::ReverseSquiggly,
                TetrominoType::TBlock,
                TetrominoType::LBlock,
                TetrominoType::ReverseLBlock,
            ];

            let mut rng = rand::rng();
            self.bag.shuffle(&mut rng);
        }

        let next_type = self.bag.pop().unwrap();

        Tetromino::new(next_type)
    }

    fn is_valid_pos(&self, x: i32, y: i32, piece: &Tetromino) -> bool {
        for offset in piece.blocks.iter() {
            let nx = x + offset.0;
            let ny = y + offset.1;

            if nx < 0 || nx >= GAME_BOARD_W as i32 {
                return false;
            }

            if ny >= GAME_BOARD_H as i32 {
                return false;
            }

            if ny >= 0 && self.game_board[ny as usize][nx as usize] {
                return false;
            }
        }
        true
    }

    fn move_piece(&mut self, x: i32, y: i32) {
        if self.is_valid_pos(self.piece_x + x, self.piece_y + y, &self.active_piece) {
            self.piece_x += x;
            self.piece_y += y;
        }
    }

    fn rotate_piece(&mut self) {
        let rotated = self.active_piece.get_rotated_copy();

        let kicks = [
            (0, 0),  // no kicks
            (-1, 0), // kick 1 step to left
            (1, 0),  // kick 1 step to right
            (0, -1), // kick up
            (-2, 0), // kick 2 steps to left
            (2, 0),  // kick 2 steps to right
        ];

        for (dx, dy) in kicks.iter() {
            let nx = self.piece_x + dx;
            let ny = self.piece_y + dy;

            if self.is_valid_pos(nx, ny, &rotated) {
                self.active_piece = rotated;
                self.piece_x = nx;
                self.piece_y = ny;
                return;
            }
        }
    }

    fn drop_piece(&mut self) {
        while self.is_valid_pos(self.piece_x, self.piece_y + 1, &self.active_piece) {
            self.piece_y += 1;
        }
    }

    fn lock_piece(&mut self) {
        for offset in self.active_piece.blocks.iter() {
            let final_x = offset.0 + self.piece_x;
            let final_y = offset.1 + self.piece_y;

            if final_x >= 0 && final_y >= 0 {
                self.game_board[final_y as usize][final_x as usize] = true;
            }
        }
    }

    fn spawn_new_piece(&mut self) {
        self.active_piece = self.new_piece();

        self.piece_x = (GAME_BOARD_W / 2) as i32 - 1;
        self.piece_y = 0;

        if !self.is_valid_pos(self.piece_x, self.piece_y, &self.active_piece) {
            self.is_running = false;
        }
    }

    fn clear_line(&mut self) {
        let mut y = GAME_BOARD_H - 1;
        while y > 0 {
            let mut is_full = true;

            for x in 0..GAME_BOARD_W {
                if !self.game_board[y][x] {
                    is_full = false;
                    break;
                }
            }

            if is_full {
                for shift_y in (1..=y).rev() {
                    self.game_board[shift_y] = self.game_board[shift_y - 1];
                }

                self.game_board[0] = [false; GAME_BOARD_W];
            } else {
                y -= 1;
            }
        }
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            match key.kind {
                event::KeyEventKind::Press => match key.code {
                    KeyCode::Left => self.move_piece(-1, 0),
                    KeyCode::Right => self.move_piece(1, 0),
                    KeyCode::Up => self.rotate_piece(),
                    KeyCode::Down => self.move_piece(0, 1),
                    KeyCode::Char(' ') => self.drop_piece(),
                    KeyCode::Char('q') => self.is_running = false,
                    _ => {}
                },
                event::KeyEventKind::Repeat => match key.code {
                    KeyCode::Down => self.move_piece(0, 1),
                    _ => {}
                },
                event::KeyEventKind::Release => todo!(),
            }
        }
        Ok(())
    }

    fn update_game_win_buf(&mut self) {
        self.game_win_buf = [[0; GAME_BOARD_W]; GAME_BOARD_H];

        for x in 0..self.game_board[0].len() {
            for y in 0..self.game_board.len() {
                if self.game_board[y][x] {
                    self.game_win_buf[y][x] = 1;
                }
            }
        }

        for offset in self.active_piece.blocks.iter() {
            let screen_x = self.piece_x + offset.0;
            let screen_y = self.piece_y + offset.1;

            if (screen_x as usize) < self.game_win_buf[0].len()
                && (screen_y as usize) < self.game_win_buf.len()
            {
                self.game_win_buf[screen_y as usize][screen_x as usize] = 1;
            }
        }
    }
}

#[derive(Default, Clone)]
struct Tetromino {
    blocks: [(i32, i32); 4],
    t_type: TetrominoType,
}

impl Tetromino {
    fn new(t_type: TetrominoType) -> Self {
        match t_type {
            TetrominoType::Square => Self {
                blocks: [(0, 0), (1, 0), (0, 1), (1, 1)],
                t_type: t_type,
            },
            TetrominoType::Line => Self {
                blocks: [(-1, 0), (0, 0), (1, 0), (2, 0)],
                t_type: t_type,
            },
            TetrominoType::Squiggly => Self {
                blocks: [(-1, 0), (0, 0), (0, 1), (1, 1)],
                t_type: t_type,
            },
            TetrominoType::ReverseSquiggly => Self {
                blocks: [(1, 0), (0, 0), (0, -1), (1, -1)],
                t_type: t_type,
            },
            TetrominoType::TBlock => Self {
                blocks: [(0, -1), (-1, 0), (0, 0), (1, 0)],
                t_type: t_type,
            },
            TetrominoType::LBlock => Self {
                blocks: [(1, -1), (-1, 0), (0, 0), (1, 0)],
                t_type: t_type,
            },
            TetrominoType::ReverseLBlock => Self {
                blocks: [(-1, -1), (-1, 0), (0, 0), (1, 0)],
                t_type: t_type,
            },
        }
    }

    fn get_rotated_copy(&mut self) -> Self {
        let mut rotated = self.clone();
        if matches!(rotated.t_type, TetrominoType::Square) {
            return rotated;
        }

        for block in rotated.blocks.iter_mut() {
            let temp = block.0;
            block.0 = -block.1;
            block.1 = temp;
        }

        rotated
    }
}

enum WinType {
    Game,
    Next,
}

struct Screen {
    buffer: Vec<Vec<Pixel>>,
    game_win: Window,
    next_win: Window, // next piece preview box
}

impl Screen {
    fn new(screen_w: usize, screen_h: usize) -> Self {
        let game_pos_x = screen_w.saturating_sub(GAME_BOARD_WIN_W) / 4;
        let game_pos_y = screen_h.saturating_sub(GAME_BOARD_WIN_H) / 2;

        let next_pos_x = game_pos_x + GAME_BOARD_WIN_W + 10;
        let next_pos_y = game_pos_y + 2;

        Self {
            buffer: vec![vec![Pixel::default(); screen_w as usize]; screen_h as usize],
            game_win: Window::new(GAME_BOARD_WIN_W, GAME_BOARD_WIN_H, game_pos_x, game_pos_y),
            next_win: Window::new(NEXT_BOARD_WIN_W, NEXT_BOARD_WIN_H, next_pos_x, next_pos_y),
        }
    }

    fn clear_win(&mut self, win_type: WinType, ch: char) {
        match win_type {
            WinType::Game => self.game_win.clear(&mut self.buffer, ch),
            WinType::Next => self.next_win.clear(&mut self.buffer, ch),
        }
    }

    fn update_buf<const COLS: usize, const ROWS: usize>(
        &mut self,
        win_type: WinType,
        board: &[[u8; COLS]; ROWS],
    ) {
        let board_cols = board[0].len();
        let board_rows = board.len();

        for row in 0..board_rows {
            for col in (0..board_cols).rev() {
                if board[row][col] != 1 {
                    continue;
                }

                match win_type {
                    WinType::Game => self.game_win.add_block(&mut self.buffer, col, row),
                    WinType::Next => self.next_win.add_block(&mut self.buffer, col, row),
                }
            }
        }
    }

    fn display_text(&mut self, text: String, color: Color, posx: usize, posy: usize) {
        for (i, ch) in text.chars().enumerate() {
            if posx + i < self.buffer[0].len() && posy < self.buffer.len() {
                self.buffer[posy][posx + i] = Pixel { ch, color: color };
            }
        }
    }

    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        for (y, row) in self.buffer.iter().enumerate() {
            queue!(stdout, cursor::MoveTo(0, y as u16))?;
            for pixel in row {
                queue!(
                    stdout,
                    style::PrintStyledContent(pixel.ch.with(pixel.color))
                )?;
            }
        }
        Ok(())
    }
}

struct Window {
    width: usize,
    height: usize,
    pos_x: usize,
    pos_y: usize,
}

impl Window {
    fn new(width: usize, height: usize, posx: usize, posy: usize) -> Self {
        return Self {
            width: width,
            height: height,
            pos_x: posx,
            pos_y: posy,
        };
    }

    fn clear(&self, buf: &mut Vec<Vec<Pixel>>, ch: char) {
        let screen_w = buf[0].len();
        let screen_h = buf.len();

        for y in 0..self.height + 1 {
            let target_y = (self.pos_y + y) as usize;
            for x in 0..self.width {
                let target_x = (self.pos_x + x + y) as usize;
                if target_x < screen_w as usize && target_y < screen_h as usize {
                    buf[target_y][target_x] = Pixel {
                        ch: ch,
                        color: Color::White,
                    }
                }

                // clearing board boundary
                if x == 0 && target_x > 0 {
                    buf[target_y][target_x - 1] = Pixel::default();
                }
                if x == self.width - 1 && target_x + 1 < screen_w {
                    buf[target_y][target_x + 1] = Pixel::default();
                }
            }
        }
    }

    fn add_block(&self, buf: &mut Vec<Vec<Pixel>>, mut posx: usize, posy: usize) {
        let height = buf.len();
        let width = buf[0].len();
        if posy + 1 + self.pos_y >= height || posx + posy + 3 + self.pos_x >= width {
            return;
        }

        posx *= 3; // projecting board coordinate to screen buffer coordinate, single block in board is 3 pixels wide in the screen buffer.
        buf[posy + self.pos_y][posx + posy + self.pos_x] = Pixel {
            ch: '/',
            color: Color::Blue,
        };
        buf[posy + self.pos_y][posx + posy + 1 + self.pos_x] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };

        buf[posy + self.pos_y][posx + posy + 2 + self.pos_x] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };
        buf[posy + self.pos_y][posx + posy + 3 + self.pos_x] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };
        buf[posy + 1 + self.pos_y][posx + posy + self.pos_x] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };
        buf[posy + 1 + self.pos_y][posx + posy + 1 + self.pos_x] = Pixel {
            ch: '/',
            color: Color::DarkBlue,
        };
        buf[posy + 1 + self.pos_y][posx + posy + 2 + self.pos_x] = Pixel {
            ch: '_',
            color: Color::DarkBlue,
        };
        buf[posy + 1 + self.pos_y][posx + posy + 3 + self.pos_x] = Pixel {
            ch: '/',
            color: Color::DarkBlue,
        };
    }
}

#[derive(Clone)]
struct Pixel {
    ch: char,
    color: Color,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            ch: ' ',
            color: Color::Reset,
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    execute!(
        stdout,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let (screen_w, screen_h) = terminal::size()?;
    let mut screen = Screen::new(screen_w as usize, screen_h as usize);
    let mut state = GameState::new();
    state.spawn_new_piece();

    let mut last_fall_time = Instant::now();
    let fall_speed = Duration::from_millis(500);

    while state.is_running {
        // input
        if event::poll(Duration::from_millis(50))? {
            state.handle_input()?
        }

        // update game state
        if last_fall_time.elapsed() >= fall_speed {
            if state.is_valid_pos(state.piece_x, state.piece_y + 1, &state.active_piece) {
                state.piece_y += 1;
            } else {
                state.lock_piece();
                state.clear_line();
                state.spawn_new_piece();
            }

            last_fall_time = Instant::now();
        }
        state.update_game_win_buf();

        // update screen buffer
        screen.clear_win(WinType::Game, '_');
        screen.clear_win(WinType::Next, '`');

        screen.update_buf(WinType::Game, &state.game_win_buf);
        screen.update_buf(WinType::Next, &state.next_win_buf);

        let score_text = format!("Score: {}\t", 1200);
        let line_text = format!("Line: {}", 100);
        screen.display_text(score_text, Yellow, 7, 2);
        screen.display_text(line_text, Cyan, 7, 3);

        // render
        screen.render(&mut stdout)?;

        stdout.flush()?;
    }

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
