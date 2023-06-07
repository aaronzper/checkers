mod board;
mod actor;
mod point;

use std::io::stdout;
use board::{Side, Board};
use crossterm::Result;
use crate::actor::ActorType;

#[tokio::main]
async fn main() -> Result<()> {
    /*let mut board = board::Board::new(8, 8, None).await?;
    //board.draw().await?;

    match board.play(ActorType::Random, ActorType::Random).await.unwrap().unwrap() {
        Side::Red => println!("Red won!"),
        Side::Blue => println!("Blue won!")
    };*/

    for _ in 0..9999 {
        let mut board = Board::new(8, 8, None).await?;
        println!("{:?}", board.play(ActorType::Random, ActorType::Random).await?.unwrap());
    }

    Ok(())
}
