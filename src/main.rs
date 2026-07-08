use std::io::{self, Write};

use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize},
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};

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

    for y in 0..board_height {
        let leading_spaces = y;
        let cursor_pos_x = start_col + leading_spaces;

        queue!(stdout, cursor::MoveTo(cursor_pos_x, start_row + y))?;
        for _ in 0..board_width {
            if y % 2 == 0 {
                queue!(stdout, style::PrintStyledContent("/".magenta()))?;
            } else {
                queue!(stdout, style::PrintStyledContent("_".magenta()))?;
            }
        }
    }

    stdout.flush()?;

    std::thread::sleep(std::time::Duration::from_hours(3));

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;

    println!("cols: {} rows: {}", start_col, start_row);
    Ok(())
}
