mod board;
mod actor;
mod point;
mod side;

use std::io::stdout;
use crossterm::Result;
use actor::ActorType;
use side::Side;
use board::Board;

#[tokio::main]
async fn main() -> Result<()> {
    let mut board = Board::new(8, 8, Some(stdout())).await?;
    board.draw().await?;

    match board.play(ActorType::Random, ActorType::Random).await.unwrap().unwrap() {
        Side::Red => println!("Red won!"),
        Side::Blue => println!("Blue won!")
    };

    Ok(())
}
