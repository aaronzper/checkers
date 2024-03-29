use tokio::time::sleep;
use std::time::Duration;
use rand::Rng;
use std::collections::HashMap;

use crate::side::Side;
use crate::point::Point;
use crate::game::{Game, GameResult};

#[derive(PartialEq, Clone, Copy)]
pub enum SimulatedActorType {
    Random,
    MostKills
}

#[derive(PartialEq, Clone, Copy)]
pub enum ActorType {
    Human,
    Random,
    MostKills,
    Simulated(SimulatedActorType)
}

impl From<SimulatedActorType> for ActorType {
    fn from(input: SimulatedActorType) -> Self {
        match input {
            SimulatedActorType::Random => ActorType::Random,
            SimulatedActorType::MostKills => ActorType::MostKills
        }
    }
}

pub struct Actor {
    pub actor_type: ActorType,
    pub side: Side
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Action {
    pub from: Point,
    pub to: Point
}

#[derive(PartialEq)]
pub enum ActionResult {
    TookAction(Action),
    NoPiecesLeft
}

impl Actor {
    #[async_recursion::async_recursion] // Needed to allow async recursion for the recursive actor
    pub async fn act(&self, game: &mut Game) -> ActionResult {
        // Sleep for 100ms if this is an AI and we're connected to the terminal
        if (self.actor_type != ActorType::Human) && game.terminal_wrapper.is_some() {
            sleep(Duration::from_millis(100)).await;
        }

        let all_moves = game.board.get_all_moves(self.side);

        if all_moves.keys().len() == 0 { // There are no pieces on this side thus other side won
            return ActionResult::NoPiecesLeft;
        }

        match self.actor_type {
            ActorType::Human => {
                // Highlight all pieces
                for x in 0..game.board.width {
                    for y in 0..game.board.height {
                        let mut highlight = false;
                        if all_moves.keys().any(|pt| *pt == Point { x, y }) {
                            highlight = true;
                        }
                        game.board.state[x as usize][y as usize].highlighted = highlight;
                    }
                }

                game.terminal_wrapper.as_mut().unwrap().draw(&game.board).await.unwrap();
                println!("Select which piece you want to move");

                loop {
                    let piece_cords = game.terminal_wrapper.as_mut().unwrap().next_click(&game.board).await.unwrap();

                    let maybe_piece = game.board.state[piece_cords.x as usize][piece_cords.y as usize].piece;
                    if !self.side.piece_is_friendly(&maybe_piece) {
                        continue; // Pick a new piece if we picked a spot that doesnt have one of
                                  // our pieces
                    }

                    let valid_moves = all_moves.get(&piece_cords).unwrap().clone();
                    if valid_moves.len() == 0 {
                        continue; // Pick a new piece if the one we picked cant make any valid
                                  // moves
                    }

                    // Highlight all valid moves
                    for x in 0..game.board.width {
                        for y in 0..game.board.height {
                            game.board.state[x as usize][y as usize].highlighted = valid_moves.contains(&Point { x, y })
                        }
                    }
                    game.terminal_wrapper.as_mut().unwrap().draw(&game.board).await.unwrap();
                    println!("Select where you'd like to move the piece");

                    loop {
                        let chosen_move = game.terminal_wrapper.as_mut().unwrap().next_click(&game.board).await.unwrap();
                        if !valid_moves.contains(&chosen_move) {
                            continue; // Pick a new move if we picked a spot that isnt a valid move
                        }

                        return ActionResult::TookAction(Action { from: piece_cords, to: chosen_move });
                    }
                }
            },
            ActorType::Random => {
                let mut rng = rand::thread_rng();

                let piece_index = rng.gen_range(0..all_moves.keys().len());
                let piece = all_moves.keys().collect::<Vec<&Point>>()[piece_index];

                let moves = all_moves.get(piece).unwrap();

                let move_index = rng.gen_range(0..moves.len());
                let chosen_move = &moves[move_index];

                return ActionResult::TookAction(Action {
                    from: piece.clone(),
                    to: chosen_move.clone()
                });
            },
            ActorType::MostKills => unimplemented!(),
            ActorType::Simulated(simulated_actor) => {
                let mut futures = HashMap::new();
                all_moves.keys().for_each(|piece| {
                    all_moves.get(piece).unwrap().iter().for_each(|move_to| {
                        let action = Action {
                            from: piece.clone(),
                            to: move_to.clone()
                        };

                        let sim_game = Game {
                            board: game.board.clone(),
                            terminal_wrapper: None
                        };
                        futures.insert(action.clone(), tokio::spawn(simulate_action(sim_game, action, simulated_actor)));
                    });
                });

                let mut results = HashMap::new();
                for future in futures {
                    results.insert(future.0, future.1.await.unwrap());
                }

                let winning_results = results.iter().filter(|result| {
                    result.1.winner == self.side
                });

                let quickest_win = match winning_results.min_by_key(|result| result.1.moves) {
                    Some(x) => x,
                    None => results.iter().max_by_key(|result| result.1.moves).unwrap() // If min returns None, there are no winning games.
                                                                                        // Thus, pick the one where we lose in the most moves
                };

                return ActionResult::TookAction(quickest_win.0.clone());
            }
        }
    }
}

// Simulates the result of an action, and returns how many steps it took to get to that result
async fn simulate_action(mut game: Game, action: Action, actor_type: SimulatedActorType) -> GameResult {
    game.board.do_action(&action).await;
    game.play(actor_type.into(), actor_type.into()).await.unwrap().unwrap()
}
