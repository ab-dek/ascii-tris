use std::io::{self, Write};

use crossterm::{
    cursor, execute, queue,
    style::{self, Color, Stylize},
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};

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

    let (cols, rows) = terminal::size()?;
    let board_width = 30;
    let board_height = 20;

    let start_col = cols.saturating_sub(board_width) / 4;
    let start_row = rows.saturating_sub(board_height) / 2;

    let mut screen_buffer = vec![
        vec![
            Pixel {
                ch: ' ',
                color: Color::Reset
            };
            cols as usize
        ];
        rows as usize
    ];

    // update screen frame buffer
    for y in 0..board_height + 1 {
        let target_y = (start_row + y) as usize;
        for x in 0..board_width {
            let target_x = (start_col + x + y) as usize;
            if target_x < cols as usize && target_y < rows as usize {
                screen_buffer[target_y][target_x] = Pixel {
                    ch: '_',
                    color: Color::White,
                }
            }
        }
    }

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

    stdout.flush()?;

    std::thread::sleep(std::time::Duration::from_secs(10));

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    println!("cols: {} rows: {}", start_col, start_row);
    Ok(())
}

fn draw_block(
    screen_buffer: &mut Vec<Vec<Pixel>>,
    start_row: usize,
    start_col: usize,
    mut pos_x: usize,
    pos_y: usize,
) {
    // check if the max col/row a block occupies is overflowing
    let rows = screen_buffer.len();
    let cols = screen_buffer[0].len();
    if pos_y + 1 + start_row >= rows || pos_x + pos_y + 3 + start_col >= cols {
        return;
    }

    pos_x *= 3;
    screen_buffer[pos_y + start_row][pos_x + pos_y + start_col] = Pixel {
        ch: '/',
        color: Color::Blue,
    };
    screen_buffer[pos_y + start_row][pos_x + pos_y + 1 + start_col] = Pixel {
        ch: '\\',
        color: Color::Blue,
    };

    screen_buffer[pos_y + start_row][pos_x + pos_y + 2 + start_col] = Pixel {
        ch: '\\',
        color: Color::Blue,
    };
    screen_buffer[pos_y + start_row][pos_x + pos_y + 3 + start_col] = Pixel {
        ch: '\\',
        color: Color::Blue,
    };
    screen_buffer[pos_y + 1 + start_row][pos_x + pos_y + start_col] = Pixel {
        ch: '\\',
        color: Color::Blue,
    };
    screen_buffer[pos_y + 1 + start_row][pos_x + pos_y + 1 + start_col] = Pixel {
        ch: '/',
        color: Color::DarkBlue,
    };
    screen_buffer[pos_y + 1 + start_row][pos_x + pos_y + 2 + start_col] = Pixel {
        ch: '/',
        color: Color::DarkBlue,
    };
    screen_buffer[pos_y + 1 + start_row][pos_x + pos_y + 3 + start_col] = Pixel {
        ch: '/',
        color: Color::DarkBlue,
    };
}
