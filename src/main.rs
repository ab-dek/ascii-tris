use std::{
    io::{self, Write},
    time::Duration,
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

const GAME_BOARD_W: usize = 10;
const GAME_BOARD_H: usize = 20;
const NEXT_BOARD_W: usize = 6;
const NEXT_BOARD_H: usize = 6;
const GAME_BOARD_WIN_W: usize = GAME_BOARD_W * 3;
const GAME_BOARD_WIN_H: usize = GAME_BOARD_H;
const NEXT_BOARD_WIN_W: usize = NEXT_BOARD_W * 3;
const NEXT_BOARD_WIN_H: usize = NEXT_BOARD_H;

struct GameState {
    game_board: [[u8; GAME_BOARD_W]; GAME_BOARD_H],
    next_board: [[u8; NEXT_BOARD_W]; NEXT_BOARD_H],
    is_running: bool,
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

    // example state of a board to test if rendering works properly
    let board: [[u8; GAME_BOARD_W]; GAME_BOARD_H] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 1, 1, 0],
        [1, 1, 0, 0, 0, 0, 1, 1, 1, 0],
        [1, 1, 1, 0, 0, 1, 1, 0, 1, 1],
        [1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    ];

    let next: [[u8; NEXT_BOARD_W]; NEXT_BOARD_H] = [
        [0, 0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0, 0],
        [0, 0, 1, 1, 0, 0],
        [0, 0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0],
    ];

    let mut state = GameState {
        game_board: board,
        next_board: next,
        is_running: true,
    };

    while state.is_running {
        // input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => state.is_running = false,
                    _ => {}
                }
            }
        }

        // update game state

        // update screen buffer
        screen.clear_win(WinType::Game, '_');
        screen.clear_win(WinType::Next, '`');

        screen.update_buf(WinType::Game, &state.game_board);
        screen.update_buf(WinType::Next, &state.next_board);

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
