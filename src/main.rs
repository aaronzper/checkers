mod board;
mod actor;

use std::io::stdout;
use actor::Actor;
use crossterm::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let actor = Actor {
        actor_type: actor::ActorType::Human
    };

    let board = board::Board::new(8, 8, Some(stdout())).await.unwrap();
    board.lock().await.draw().await?;

    Ok(())
}
