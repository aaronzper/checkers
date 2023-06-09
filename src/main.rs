mod game;
mod actor;
mod point;
mod side;
mod piece;

use std::io::stdout;
use crossterm::Result;
use actor::ActorType;
use side::Side;
use game::Game;

#[tokio::main]
async fn main() -> Result<()> {
    let mut game = Game::new(8, 8, Some(stdout()))?;
    game.play(ActorType::Random, ActorType::Random).await.unwrap().unwrap();

    Ok(())
}
