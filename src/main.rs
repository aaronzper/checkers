mod board;
mod actor;
mod point;

use std::io::stdout;
use board::Side;
use crossterm::Result;
use crate::actor::ActorType;

#[tokio::main]
async fn main() -> Result<()> {
    let mut board = board::Board::new(8, 8, Some(stdout())).await?;
    board.draw().await?;

    match board.play(ActorType::Random, ActorType::Random).await.unwrap().unwrap() {
        Side::Red => println!("Red won!"),
        Side::Blue => println!("Blue won!")
    };

    Ok(())
}
