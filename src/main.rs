mod board;
mod actor;

use std::io::stdout;
use crossterm::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut board = board::Board::new(8, 8, Some(stdout())).await?;
    board.draw().await?;

    board.play().await?;

    Ok(())
}
