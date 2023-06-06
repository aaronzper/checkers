use std::todo;
use crate::board::{Board, Side, Piece};
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
    pub async fn act(&self, board: &mut Board) -> ActionResult {
        match self.actor_type {
            ActorType::Human => {
                let mut is_piece_this_side = false;
                // Highlight all pieces
                for x in 0..board.width {
                    for y in 0..board.height {
                        let piece = board.state[x as usize][y as usize].piece;
                        let mut highlight = false;
                        if self.side.piece_is_friendly(&piece) {
                            is_piece_this_side = true;
                            if board.valid_moves(&Point {x, y}, piece.unwrap()).unwrap().len() != 0 {
                                highlight = true;
                            }
                        }
                        board.state[x as usize][y as usize].highlighted = highlight;
                    }
                }

                if !is_piece_this_side { // There are no pieces on this side thus other side won
                    return ActionResult::NoPiecesLeft;
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
            _ => todo!()
        }
    }
}
