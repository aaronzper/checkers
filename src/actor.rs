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
pub enum ActionResult {
    TookAction,
    NoPiecesLeft
}

impl Actor {
    fn piece_is_friendly(&self, piece: &Option<Piece>) -> bool {
        match piece {
            None => false,
            Some(x) => {
                x.side == self.side
            }
        }
    }

    fn piece_is_hostile(&self, piece: &Option<Piece>) -> bool {
        match piece {
            None => false,
            Some(x) => {
                x.side != self.side
            }
        }
    }

    pub async fn act(&self, board: &mut Board) -> ActionResult {
        match self.actor_type {
            ActorType::Human => {
                let mut is_piece_this_side = false;
                // Highlight all pieces
                for x in 0..board.width {
                    for y in 0..board.height {
                        let piece = board.state[x as usize][y as usize].piece;
                        let mut highlight = false;
                        if self.piece_is_friendly(&piece) {
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
                    if !self.piece_is_friendly(&maybe_piece) {
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

                        // Crown the piece, if applicable
                        let crowned = if piece.crowned {
                            true // If it's already crowned, keep it that way
                        }
                        else {
                            if (self.side == Side::Red && chosen_move.y == board.height - 1) || (self.side == Side::Blue && chosen_move.y == 0) {
                                true
                            }
                            else {
                                false
                            }
                        };

                        // Actually move the piece
                        board.state[piece_cords.x as usize][piece_cords.y as usize].piece = None;
                        board.state[chosen_move.x as usize][chosen_move.y as usize].piece = Some(Piece { side: self.side, crowned });

                        // Find out the bigger/smaller x and y from the source/destination for below
                        let bigger_x;
                        let smaller_x;
                        let bigger_y;
                        let smaller_y;
                        if piece_cords.x > chosen_move.x {
                            bigger_x = piece_cords.x;
                            smaller_x = chosen_move.x;
                        }
                        else {
                            bigger_x = chosen_move.x;
                            smaller_x = piece_cords.x;
                        }
                        if piece_cords.y > chosen_move.y {
                            bigger_y = piece_cords.y;
                            smaller_y = chosen_move.y;
                        }   
                        else {
                            bigger_y = chosen_move.y;
                            smaller_y = piece_cords.y;
                        }

                        // Remove enemy pieces between source and destination
                        for x in smaller_x..bigger_x {
                            for y in smaller_y..bigger_y {
                                if piece_cords.x.abs_diff(x as u8) == piece_cords.y.abs_diff(y as u8) {
                                    if self.piece_is_hostile(&board.state[x as usize][y as usize].piece) {
                                        board.state[x as usize][y as usize].piece = None;
                                    }
                                }
                            }
                        }

                        board.draw().await.unwrap();
                        return ActionResult::TookAction;
                    }
                }
            },
            _ => todo!()
        }
    }
}
