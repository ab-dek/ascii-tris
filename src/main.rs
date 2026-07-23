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
        Color::{self, Cyan, DarkGrey, Red, Yellow},
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

#[derive(Default, Clone, Debug)]
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

#[derive(Default, Debug)]
struct GameState {
    active_piece: Tetromino,
    piece_x: i8,
    piece_y: i8,
    next_piece: Tetromino,
    game_board: [[BlockColor; GAME_BOARD_W]; GAME_BOARD_H],

    lock_timer: Option<Instant>,
    lock_reset_limit: u8,

    bag: Vec<TetrominoType>,

    speed: u64,
    score: u32,
    lines: u32,
    level: u32,

    game_win_buf: [[BlockColor; GAME_BOARD_W]; GAME_BOARD_H],
    next_win_buf: [[BlockColor; NEXT_BOARD_W]; NEXT_BOARD_H],
    is_running: bool,
}

impl GameState {
    fn new() -> Self {
        Self {
            speed: 800,
            level: 1,
            lock_reset_limit: 15,
            is_running: true,
            ..Default::default()
        }
    }

    fn new_piece(&mut self) -> Tetromino {
        if self.bag.is_empty() {
            self.bag.extend([
                TetrominoType::Square,
                TetrominoType::Line,
                TetrominoType::Squiggly,
                TetrominoType::ReverseSquiggly,
                TetrominoType::TBlock,
                TetrominoType::LBlock,
                TetrominoType::ReverseLBlock,
            ]);

            let mut rng = rand::rng();
            self.bag.shuffle(&mut rng);
        }

        let next_type = self.bag.pop().unwrap();

        Tetromino::new(next_type)
    }

    fn is_valid_pos(&self, x: i8, y: i8, piece: &Tetromino) -> bool {
        for offset in piece.blocks.iter() {
            let nx = x + offset.0;
            let ny = y + offset.1;

            if nx < 0 || nx >= GAME_BOARD_W as i8 {
                return false;
            }

            if ny < 0 || ny >= GAME_BOARD_H as i8 {
                return false;
            }

            if !matches!(self.game_board[ny as usize][nx as usize], BlockColor::None) {
                return false;
            }
        }
        true
    }

    fn move_piece(&mut self, x: i8, y: i8) {
        if self.is_valid_pos(self.piece_x + x, self.piece_y + y, &self.active_piece) {
            self.piece_x += x;
            self.piece_y += y;

            if self.piece_landed() && self.lock_reset_limit > 0 {
                self.lock_timer = Some(Instant::now());
                self.lock_reset_limit -= 1;
            }
        }
    }

    fn rotate_piece(&mut self) {
        let rotated = self.active_piece.get_rotated_copy();

        let kicks = [
            (0, 0),  // no kicks
            (-1, 0), // kick 1 step to left
            (1, 0),  // kick 1 step to right
            (0, -1), // kick up
            (0, 1),  // kick down
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

                if self.piece_landed() && self.lock_reset_limit > 0 {
                    self.lock_timer = Some(Instant::now());
                    self.lock_reset_limit -= 1;
                }

                return;
            }
        }
    }

    fn drop_piece(&mut self) {
        while self.is_valid_pos(self.piece_x, self.piece_y + 1, &self.active_piece) {
            self.piece_y += 1;
        }
        self.lock_piece();
    }

    fn lock_piece(&mut self) {
        for offset in self.active_piece.blocks.iter() {
            let final_x = offset.0 + self.piece_x;
            let final_y = offset.1 + self.piece_y;

            if final_x >= 0 && final_y >= 0 {
                self.game_board[final_y as usize][final_x as usize] = self.active_piece.color;
            }
        }
    }

    fn spawn_new_piece(&mut self) {
        self.active_piece = self.next_piece.clone();
        self.next_piece = self.new_piece();

        self.piece_x = (GAME_BOARD_W / 2) as i8 - 1;
        self.piece_y = 0;

        // pushing down the piece so that its entirely visible
        for dy in 0..=2 {
            let ny = self.piece_y + dy;

            if self.is_valid_pos(self.piece_x, ny, &self.active_piece) {
                self.piece_y = ny;
                return;
            }
        }

        self.is_running = false; // game over
    }

    fn piece_landed(&mut self) -> bool {
        !self.is_valid_pos(self.piece_x, self.piece_y + 1, &self.active_piece)
    }

    fn clear_line(&mut self) {
        let mut cleared = 0;
        let mut y = GAME_BOARD_H - 1;

        while y > 0 {
            let mut is_full = true;

            for x in 0..GAME_BOARD_W {
                if matches!(self.game_board[y][x], BlockColor::None) {
                    is_full = false;
                    break;
                }
            }

            if is_full {
                for shift_y in (1..=y).rev() {
                    self.game_board[shift_y] = self.game_board[shift_y - 1];
                }

                self.game_board[0] = [BlockColor::None; GAME_BOARD_W];
                cleared += 1;
            } else {
                y -= 1;
            }
        }

        if cleared > 0 {
            self.score += match cleared {
                1 => 100 * self.level,
                2 => 300 * self.level,
                3 => 500 * self.level,
                4 => 800 * self.level,
                _ => 0,
            };
            self.lines += cleared;

            self.level = self.lines / 10 + 1;

            self.speed = match self.level {
                1 => 800,
                2 => 720,
                3 => 630,
                4 => 550,
                5 => 470,
                6 => 380,
                7 => 300,
                8 => 220,
                9 => 130,
                10 => 100,
                11 => 80,
                _ => 0,
            };
        }
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Left => self.move_piece(-1, 0),
                KeyCode::Right => self.move_piece(1, 0),
                KeyCode::Up => self.rotate_piece(),
                KeyCode::Down => self.move_piece(0, 1),
                KeyCode::Char(' ') => self.drop_piece(),
                KeyCode::Char('q') => self.is_running = false,
                _ => {}
            }
        }
        Ok(())
    }

    fn get_ghost_pos_y(&self) -> i8 {
        let mut ghost_pos_y = self.piece_y;

        while self.is_valid_pos(self.piece_x, ghost_pos_y + 1, &self.active_piece) {
            ghost_pos_y += 1;
        }

        ghost_pos_y
    }

    fn update_game_win_buf(&mut self) {
        self.game_win_buf = [[BlockColor::None; GAME_BOARD_W]; GAME_BOARD_H];

        // add locked pieces
        for x in 0..self.game_board[0].len() {
            for y in 0..self.game_board.len() {
                if !matches!(self.game_board[y][x], BlockColor::None) {
                    self.game_win_buf[y][x] = self.game_board[y][x];
                }
            }
        }

        // add ghost piece
        // TODO: refactor this out into a fn, something like draw_piece(tet, x, y, win)
        for offset in self.active_piece.blocks.iter() {
            let screen_x = self.piece_x + offset.0;
            let screen_y = self.get_ghost_pos_y() + offset.1;

            if (screen_x as usize) < self.game_win_buf[0].len()
                && (screen_y as usize) < self.game_win_buf.len()
            {
                self.game_win_buf[screen_y as usize][screen_x as usize] = BlockColor::DarkGrey;
            }
        }

        // add active piece
        for offset in self.active_piece.blocks.iter() {
            let screen_x = self.piece_x + offset.0;
            let screen_y = self.piece_y + offset.1;

            if (screen_x as usize) < self.game_win_buf[0].len()
                && (screen_y as usize) < self.game_win_buf.len()
            {
                self.game_win_buf[screen_y as usize][screen_x as usize] = self.active_piece.color;
            }
        }
    }

    fn update_next_win_buf(&mut self) {
        self.next_win_buf = [[BlockColor::None; NEXT_BOARD_W]; NEXT_BOARD_H];
        let pos_x = NEXT_BOARD_W / 3;
        let pos_y = NEXT_BOARD_H / 3;

        for offset in self.next_piece.blocks.iter() {
            let screen_x = (pos_x as i8 + offset.0) as usize;
            let screen_y = (pos_y as i8 + offset.1) as usize;

            if (screen_x) < self.next_win_buf[0].len() && (screen_y) < self.next_win_buf.len() {
                self.next_win_buf[screen_y][screen_x] = self.next_piece.color;
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
struct Tetromino {
    blocks: [(i8, i8); 4],
    t_type: TetrominoType,
    color: BlockColor,
}

impl Tetromino {
    fn new(t_type: TetrominoType) -> Self {
        match t_type {
            TetrominoType::Square => Self {
                blocks: [(0, 0), (1, 0), (0, 1), (1, 1)],
                t_type,
                color: BlockColor::Blue,
            },
            TetrominoType::Line => Self {
                blocks: [(-1, 0), (0, 0), (1, 0), (2, 0)],
                t_type,
                color: BlockColor::Green,
            },
            TetrominoType::Squiggly => Self {
                blocks: [(-1, 0), (0, 0), (0, 1), (1, 1)],
                t_type,
                color: BlockColor::Cyan,
            },
            TetrominoType::ReverseSquiggly => Self {
                blocks: [(-1, 1), (0, 1), (0, 0), (1, 0)],
                t_type,
                color: BlockColor::Magenta,
            },
            TetrominoType::TBlock => Self {
                blocks: [(0, -1), (-1, 0), (0, 0), (1, 0)],
                t_type,
                color: BlockColor::Yellow,
            },
            TetrominoType::LBlock => Self {
                blocks: [(1, -1), (-1, 0), (0, 0), (1, 0)],
                t_type,
                color: BlockColor::Red,
            },
            TetrominoType::ReverseLBlock => Self {
                blocks: [(-1, -1), (-1, 0), (0, 0), (1, 0)],
                t_type,
                color: BlockColor::Blue,
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
        let game_pos_x = screen_w.saturating_sub(GAME_BOARD_WIN_W) / 3;
        let game_pos_y = screen_h.saturating_sub(GAME_BOARD_WIN_H) / 2;

        let next_pos_x = game_pos_x + GAME_BOARD_WIN_W + 10;
        let next_pos_y = game_pos_y + 2;

        Self {
            buffer: vec![vec![Pixel::default(); screen_w]; screen_h],
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
        board: &[[BlockColor; COLS]; ROWS],
    ) {
        let board_cols = board[0].len();
        let board_rows = board.len();

        for row in 0..board_rows {
            for col in (0..board_cols).rev() {
                if matches!(board[row][col], BlockColor::None) {
                    continue;
                }

                match win_type {
                    WinType::Game => {
                        self.game_win
                            .add_block(&mut self.buffer, col, row, board[row][col])
                    }
                    WinType::Next => {
                        self.next_win
                            .add_block(&mut self.buffer, col, row, board[row][col])
                    }
                }
            }
        }
    }

    fn display_text(&mut self, text: String, color: Color, posx: usize, posy: usize) {
        for (i, ch) in text.chars().enumerate() {
            if posx + i < self.buffer[0].len() && posy < self.buffer.len() {
                self.buffer[posy][posx + i] = Pixel { ch, color };
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
    fn new(width: usize, height: usize, pos_x: usize, pos_y: usize) -> Self {
        Self {
            width,
            height,
            pos_x,
            pos_y,
        }
    }

    fn clear(&self, buf: &mut Vec<Vec<Pixel>>, ch: char) {
        let screen_w = buf[0].len();
        let screen_h = buf.len();

        for y in 0..self.height + 1 {
            let target_y = self.pos_y + y;
            for x in 0..self.width {
                let target_x = self.pos_x + x + y;
                if target_x < screen_w && target_y < screen_h {
                    buf[target_y][target_x] = Pixel {
                        ch,
                        color: Color::DarkGrey,
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

    fn add_block(
        &self,
        buf: &mut Vec<Vec<Pixel>>,
        mut posx: usize,
        posy: usize,
        color: BlockColor,
    ) {
        let height = buf.len();
        let width = buf[0].len();
        if posy + 1 + self.pos_y >= height || posx + posy + 3 + self.pos_x >= width {
            return;
        }

        let (bright, dark) = BlockColor::get_color(color);

        posx *= 3; // projecting board coordinate to screen buffer coordinate, single block in board is 3 pixels wide in the screen buffer.
        buf[posy + self.pos_y][posx + posy + self.pos_x] = Pixel {
            ch: '/',
            color: bright,
        };
        buf[posy + self.pos_y][posx + posy + 1 + self.pos_x] = Pixel {
            ch: '\\',
            color: bright,
        };

        buf[posy + self.pos_y][posx + posy + 2 + self.pos_x] = Pixel {
            ch: '\\',
            color: bright,
        };
        buf[posy + self.pos_y][posx + posy + 3 + self.pos_x] = Pixel {
            ch: '\\',
            color: bright,
        };
        buf[posy + 1 + self.pos_y][posx + posy + self.pos_x] = Pixel {
            ch: '\\',
            color: bright,
        };
        buf[posy + 1 + self.pos_y][posx + posy + 1 + self.pos_x] = Pixel {
            ch: '/',
            color: dark,
        };
        buf[posy + 1 + self.pos_y][posx + posy + 2 + self.pos_x] = Pixel {
            ch: '_',
            color: dark,
        };
        buf[posy + 1 + self.pos_y][posx + posy + 3 + self.pos_x] = Pixel {
            ch: '/',
            color: dark,
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

#[derive(Default, Debug, Clone, Copy)]
enum BlockColor {
    #[default]
    None,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    DarkGrey,
}

impl BlockColor {
    fn get_color(self) -> (Color, Color) {
        match self {
            BlockColor::None => (Color::Reset, Color::Reset),
            BlockColor::Red => (Color::Red, Color::DarkRed),
            BlockColor::Green => (Color::Green, Color::DarkGreen),
            BlockColor::Blue => (Color::Blue, Color::DarkBlue),
            BlockColor::Yellow => (Color::Yellow, Color::DarkYellow),
            BlockColor::Magenta => (Color::Magenta, Color::DarkMagenta),
            BlockColor::Cyan => (Color::Cyan, Color::DarkCyan),
            BlockColor::DarkGrey => (Color::DarkGrey, Color::DarkGrey),
        }
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    handle_panic();

    execute!(
        stdout,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;

    let (screen_w, screen_h) = terminal::size()?;
    let mut screen = Screen::new(screen_w as usize, screen_h as usize);
    let mut state = GameState::new();
    state.next_piece = state.new_piece();
    state.spawn_new_piece();

    let mut last_fall_time = Instant::now();
    let mut fall_speed = Duration::from_millis(state.speed);
    let lock_delay = Duration::from_millis(500);

    while state.is_running {
        // -- INPUT --
        if event::poll(Duration::from_millis(50))? {
            state.handle_input()?
        }

        // -- UPDATE GAME STATE --
        // check lock delay
        if state.piece_landed() {
            match state.lock_timer {
                Some(timer) => {
                    if timer.elapsed() >= lock_delay {
                        state.lock_piece();
                        state.clear_line();
                        state.spawn_new_piece();

                        fall_speed = Duration::from_millis(state.speed);
                        last_fall_time = Instant::now();
                        state.lock_timer = None;
                        state.lock_reset_limit = 15;
                    }
                }
                None => state.lock_timer = Some(Instant::now()),
            }
        } else {
            // piece start falling again(goes off a ledge)
            state.lock_timer = None;
        }

        // update gravity
        if !state.piece_landed() && last_fall_time.elapsed() >= fall_speed {
            state.piece_y += 1;
            last_fall_time = Instant::now();
        }

        // update windows
        state.update_game_win_buf();
        state.update_next_win_buf();

        // -- UPDATE SCREEN BUFFER --
        screen.clear_win(WinType::Game, '_');
        screen.clear_win(WinType::Next, '`');

        screen.update_buf(WinType::Game, &state.game_win_buf);
        screen.update_buf(WinType::Next, &state.next_win_buf);

        let score_text = format!("Score: {}\t", state.score);
        let line_text = format!("Lines: {}\t", state.lines);
        let level_text = format!("Level: {}", state.level);
        let help_text = "quit - q | left/right - 󰍞/󰍟 | down -  | drop - space".to_string();

        let score_len = score_text.len();
        let line_len = line_text.len();

        screen.display_text(score_text, Yellow, screen_w as usize / 3, 1);
        screen.display_text(line_text, Cyan, screen_w as usize / 3 + score_len, 1);
        screen.display_text(
            level_text,
            Red,
            screen_w as usize / 3 + score_len + line_len,
            1,
        );
        screen.display_text(
            help_text,
            DarkGrey,
            screen_w as usize / 3,
            screen_h as usize - 2,
        );

        // -- RENDER --
        screen.render(&mut stdout)?;

        stdout.flush()?;
    }

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    println!("-- GAME OVER --");
    println!("Score: {}", state.score);
    println!("Lines Cleared: {}", state.lines);
    println!("Level: {}", state.level);

    Ok(())
}

fn handle_panic() {
    std::panic::set_hook(Box::new(|panic_hook_info| {
        let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        eprintln!("panic: {}", panic_hook_info)
    }));
}
