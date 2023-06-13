mod game;
mod actor;
mod point;
mod side;
mod piece;

use std::io::stdout;
use crossterm::Result;
use actor::ActorType;
use actor::RecursiveActorType;
use side::Side;
use game::Game;

#[tokio::main]
async fn main() -> Result<()> {
    let mut game = Game::new(8, 8, Some(stdout()))?;
    let result = game.play(ActorType::Recursive(RecursiveActorType::Random), ActorType::Human).await.unwrap().unwrap();
    drop(game);

    println!("{:?}", result);

    Ok(())
}
