mod game;
mod actor;
mod point;
mod side;
mod piece;

use std::env::args;
use std::io::stdout;
use crossterm::Result;
use actor::ActorType;
use actor::SimulatedActorType;
use side::Side;
use game::Game;

fn str_to_actor(input: String) -> Result<ActorType> {
    match input.to_uppercase().as_str() {
        "S" => Ok(ActorType::Simulated(SimulatedActorType::Random)),
        "R" => Ok(ActorType::Random),
        "H" => Ok(ActorType::Human),
        _ => Err(crossterm::ErrorKind::new(std::io::ErrorKind::InvalidData, "Invalid actor type code"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = args(); // Save a copy of the args
    args.next(); // Discard the first arg since its just the command
    let width = args.next().expect("Please provide a width").parse().expect("Please provide a valid width");
    let height = args.next().expect("Please provide a height").parse().expect("Please provide a valid height");

    if width < 3 { panic!("Width must be at least 3"); }
    if height < 7 { panic!("Height must be at least 7"); }

    let red_actor_str = args.next().expect("Please provide an actor type for the Red player ([h]uman, [r]andom, or [s]mart)");
    let blue_actor_str = args.next().expect("Please provide an actor type for the Blue player ([h]uman, [r]andom, or [s]mart)");
    let red_actor = str_to_actor(red_actor_str).unwrap();
    let blue_actor = str_to_actor(blue_actor_str).unwrap();

    let mut game = Game::new(width, height, Some(stdout()))?;
    let result = game.play(red_actor, blue_actor).await.unwrap().unwrap();
    drop(game); // Drop the game object to restore the terminal to normal

    println!("{:?}", result);

    Ok(())
}
