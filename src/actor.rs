use std::thread::sleep_ms;
use std::todo;
use rand::Rng;

use crate::side::Side;
use crate::point::Point;
use crate::game::{Game, GameResult};

#[derive(PartialEq, Clone, Copy)]
pub enum RecursiveActorType {
    Random,
    MostKills
}

#[derive(PartialEq, Clone, Copy)]
pub enum ActorType {
    Human,
    Random,
    MostKills,
    Recursive(RecursiveActorType)
}

impl From<RecursiveActorType> for ActorType {
    fn from(old: RecursiveActorType) -> Self {
        match old {
            RecursiveActorType::Random => ActorType::Random,
            RecursiveActorType::MostKills => ActorType::MostKills
        }
    }
}

pub struct Actor {
    pub actor_type: ActorType,
    pub side: Side
}

#[derive(PartialEq)]
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
    pub async fn act(&self, game: &mut Game) -> ActionResult {
        // Sleep for 100ms if this is an AI and we're connected to the terminal
        if self.actor_type != ActorType::Human && game.terminal_wrapper.is_some() {
            sleep_ms(300);
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
            ActorType::Recursive(recursive_actor) => {
                let mut futures = Vec::new();
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
                        futures.push(simulate_action(sim_game, action, recursive_actor));
                    });
                });

                todo!()
            }
            _ => todo!()
        }
    }
}

// Simulates the result of an action, and returns how many steps it took to get to that result
async fn simulate_action(mut game: Game, action: Action, actor_type: RecursiveActorType) -> GameResult {
    game.board.do_action(&action).await;
    game.play(actor_type.into(), actor_type.into()).await.unwrap().unwrap()
}
