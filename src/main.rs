mod board;
mod actor;
mod point;

use std::io::stdout;
use crossterm::Result;
use crate::actor::ActorType;

#[tokio::main]
async fn main() -> Result<()> {
    let mut board = board::Board::new(8, 8, Some(stdout())).await?;
    board.draw().await?;

    board.play(ActorType::Human, ActorType::Random).await?;

    Ok(())
}
