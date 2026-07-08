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
    let board_width = 40;
    let board_height = 30;

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
    for y in 0..board_height {
        let target_y = (start_row + y) as usize;
        for x in 0..board_width {
            let target_x = (start_col + x + y) as usize;
            if target_x < cols as usize && target_y < rows as usize {
                if target_x % 2 == 0 {
                    screen_buffer[target_y][target_x] = Pixel {
                        ch: '\\',
                        color: Color::Red,
                    }
                } else {
                    screen_buffer[target_y][target_x] = Pixel {
                        ch: '_',
                        color: Color::Red,
                    }
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
