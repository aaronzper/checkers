mod board;

use std::io::stdout;
use crossterm::Result;

fn main() -> Result<()> {
    let mut board = board::Board {
        width: 8,
        height: 8,
        terminal: stdout(),
        state: Vec::default()
    };

    board.draw()?;

    loop {}

    Ok(())
}
