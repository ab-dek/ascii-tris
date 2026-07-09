use std::io::{self, Write};

use crossterm::{
    cursor, execute, queue,
    style::{self, Color, Print, Stylize},
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};

struct Window {
    window_width: usize,
    window_height: usize,
    posx: usize,
    posy: usize,
}

impl Window {
    fn new(width: u16, height: u16, posx: u16, posy: u16) -> Self {
        return Self {
            window_width: width as usize,
            window_height: height as usize,
            posx: posx as usize,
            posy: posy as usize,
        };
    }

    fn clear(&self, buf: &mut Vec<Vec<Pixel>>, ch: char) {
        let screen_w = buf[0].len();
        let screen_h = buf.len();

        for y in 0..self.window_height + 1 {
            let target_y = (self.posy + y) as usize;
            for x in 0..self.window_width {
                let target_x = (self.posx + x + y) as usize;
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
        if posy + 1 + self.posy >= height || posx + posy + 3 + self.posx >= width {
            return;
        }

        posx *= 3;
        buf[posy + self.posy][posx + posy + self.posx] = Pixel {
            ch: '/',
            color: Color::Blue,
        };
        buf[posy + self.posy][posx + posy + 1 + self.posx] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };

        buf[posy + self.posy][posx + posy + 2 + self.posx] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };
        buf[posy + self.posy][posx + posy + 3 + self.posx] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };
        buf[posy + 1 + self.posy][posx + posy + self.posx] = Pixel {
            ch: '\\',
            color: Color::Blue,
        };
        buf[posy + 1 + self.posy][posx + posy + 1 + self.posx] = Pixel {
            ch: '/',
            color: Color::DarkBlue,
        };
        buf[posy + 1 + self.posy][posx + posy + 2 + self.posx] = Pixel {
            ch: '_',
            color: Color::DarkBlue,
        };
        buf[posy + 1 + self.posy][posx + posy + 3 + self.posx] = Pixel {
            ch: '/',
            color: Color::DarkBlue,
        };
    }

    fn update_buf(&self, buf: &mut Vec<Vec<Pixel>>, board: Vec<Vec<usize>>) {
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

    let (screen_width, screen_height) = terminal::size()?;

    let mut screen_buffer = vec![
        vec![
            Pixel {
                ch: ' ',
                color: Color::Reset
            };
            screen_width as usize
        ];
        screen_height as usize
    ];

    let window_width = 30;
    let window_height = 20;
    let posx = screen_width.saturating_sub(window_width) / 4;
    let posy = screen_height.saturating_sub(window_height) / 2;
    let game_window = Window::new(window_width, window_height, posx, posy);

    // next piece preview box
    let next_width = 18;
    let next_height = 6;
    let posx = screen_width.saturating_sub(next_width) * 11 / 20;
    let posy = screen_height.saturating_sub(next_height) / 3;
    let next_preview_window = Window::new(next_width, next_height, posx, posy);

    // update screen frame buffer
    game_window.clear(&mut screen_buffer, '_');
    next_preview_window.clear(&mut screen_buffer, '`');

    // example state of a board to test if rendering works properly
    let board: Vec<Vec<usize>> = vec![
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 0],
        vec![1, 1, 0, 0, 0, 0, 1, 1, 1, 0],
        vec![1, 1, 1, 0, 0, 1, 1, 0, 1, 1],
        vec![1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    ];

    game_window.update_buf(&mut screen_buffer, board);

    next_preview_window.add_block(&mut screen_buffer, 3, 1);
    next_preview_window.add_block(&mut screen_buffer, 3, 2);
    next_preview_window.add_block(&mut screen_buffer, 2, 2);
    next_preview_window.add_block(&mut screen_buffer, 2, 3);

    // render the buffer to the terminal
    for (y, row) in screen_buffer.iter().enumerate() {
        queue!(stdout, cursor::MoveTo(0, y as u16))?;
        for pixel in row {
            queue!(
                stdout,
                style::PrintStyledContent(pixel.ch.with(pixel.color))
            )?;
        }
    }

    display_score(&mut stdout)?;

    stdout.flush()?;

    std::thread::sleep(std::time::Duration::from_secs(10));

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    println!("cols: {} rows: {}", posx, posy);
    Ok(())
}

fn display_score(stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(stdout, Print("score: 1200\t"), Print("lines: 10"))?;
    Ok(())
}
