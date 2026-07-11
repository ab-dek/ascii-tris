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

        posx *= 3; // translating board coordinate to rendering grid coordinate, single block in board holds 3 pixels along the x axis.
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

    fn update_buf<const COLS: usize, const ROWS: usize>(
        &self,
        buf: &mut Vec<Vec<Pixel>>,
        board: &[[u8; COLS]; ROWS],
    ) {
        let board_cols = board[0].len();
        let board_rows = board.len();

        for row in 0..board_rows {
            for col in (0..board_cols).rev() {
                if board[row][col] == 1 {
                    self.add_block(buf, col, row);
                }
            }
        }
    }
}

#[derive(Clone)]
struct Pixel {
    ch: char,
    color: Color,
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

    let mut screen_buffer = vec![
        vec![
            Pixel {
                ch: ' ',
                color: Color::Reset
            };
            screen_w as usize
        ];
        screen_h as usize
    ];

    let posx = (screen_w as usize).saturating_sub(GAME_BOARD_WIN_W) / 4;
    let posy = (screen_h as usize).saturating_sub(GAME_BOARD_WIN_H) / 2;
    let game_window = Window::new(GAME_BOARD_WIN_W, GAME_BOARD_WIN_H, posx, posy);

    // next piece preview box
    let posx = (screen_w as usize).saturating_sub(NEXT_BOARD_WIN_W) * 11 / 20;
    let posy = (screen_h as usize).saturating_sub(NEXT_BOARD_WIN_H) / 3;
    let next_preview_window = Window::new(NEXT_BOARD_WIN_W, NEXT_BOARD_WIN_H, posx, posy);

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
        game_window.clear(&mut screen_buffer, '_');
        next_preview_window.clear(&mut screen_buffer, '`');

        game_window.update_buf(&mut screen_buffer, &state.game_board);
        next_preview_window.update_buf(&mut screen_buffer, &state.next_board);

        display_score(&mut screen_buffer, 3, 1, 1200, 10);

        // render
        for (y, row) in screen_buffer.iter().enumerate() {
            queue!(stdout, cursor::MoveTo(0, y as u16))?;
            for pixel in row {
                queue!(
                    stdout,
                    style::PrintStyledContent(pixel.ch.with(pixel.color))
                )?;
            }
        }

        stdout.flush()?;
    }

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

// TODO: update this function to be display_text(text: String, color: Color, posx, posy)
fn display_score(buf: &mut Vec<Vec<Pixel>>, posx: usize, posy: usize, score: u16, line: u16) {
    let score_text = format!("Score: {}\t", score);
    let line_text = format!("Line: {}", line);

    for (i, ch) in score_text.chars().enumerate() {
        if posx + i < buf[0].len() && posy < buf.len() {
            buf[posy][posx + i] = Pixel { ch, color: Yellow };
        }
    }

    let offset = score_text.len();
    for (i, ch) in line_text.chars().enumerate() {
        if posx + i + offset < buf[0].len() && posy < buf.len() {
            buf[posy][posx + i + offset] = Pixel { ch, color: Cyan };
        }
    }
}
