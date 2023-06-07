use std::collections::HashMap;
use std::thread::sleep_ms;
use std::todo;
use rand::Rng;

use crate::board::{Board, Side};
use crate::point::Point;

#[derive(PartialEq)]
pub enum RecursiveActorType {
    Random,
    MostKills
}

#[derive(PartialEq)]
pub enum ActorType {
    Human,
    Random,
    MostKills,
    Recursive(RecursiveActorType)
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
    fn get_all_moves(&self, board: &Board) -> HashMap<Point, Vec<Point>> {
        let mut all_moves = HashMap::new();
        for x in 0..board.width {
            for y in 0..board.height {
                let piece = board.state[x as usize][y as usize].piece;
                if self.side.piece_is_friendly(&piece) {
                    let moves = board.valid_moves(&Point {x, y}, piece.unwrap()).unwrap();
                    if moves.len() != 0 {
                        all_moves.insert(Point { x, y }, moves);
                    }
                }
            }
        }

        all_moves
    }

    pub async fn act(&self, board: &mut Board) -> ActionResult {
        // Sleep for 100ms if this is an AI and we're connected to the terminal
        if self.actor_type != ActorType::Human && board.terminal.is_some() {
            sleep_ms(300);
        }

        let all_moves = self.get_all_moves(&board);

        if all_moves.keys().len() == 0 { // There are no pieces on this side thus other side won
            return ActionResult::NoPiecesLeft;
        }

        match self.actor_type {
            ActorType::Human => {
                // Highlight all pieces
                for x in 0..board.width {
                    for y in 0..board.height {
                        let mut highlight = false;
                        if all_moves.keys().any(|pt| *pt == Point { x, y }) {
                            highlight = true;
                        }
                        board.state[x as usize][y as usize].highlighted = highlight;
                    }
                }

                board.draw().await.unwrap();
                println!("Select which piece you want to move");

                loop {
                    let piece_cords = board.next_click().await.unwrap();

                    let maybe_piece = board.state[piece_cords.x as usize][piece_cords.y as usize].piece;
                    if !self.side.piece_is_friendly(&maybe_piece) {
                        continue; // Pick a new piece if we picked a spot that doesnt have one of
                                  // our pieces
                    }

                    let piece = maybe_piece.unwrap();

                    let valid_moves = board.valid_moves(&piece_cords, piece).unwrap();
                    if valid_moves.len() == 0 {
                        continue; // Pick a new piece if the one we picked cant make any valid
                                  // moves
                    }

                    // Highlight all valid moves
                    for x in 0..board.width {
                        for y in 0..board.height {
                            board.state[x as usize][y as usize].highlighted = valid_moves.contains(&Point { x, y })
                        }
                    }
                    board.draw().await.unwrap();
                    println!("Select where you'd like to move the piece");

                    loop {
                        let chosen_move = board.next_click().await.unwrap();
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
            _ => todo!()
        }
    }
}
