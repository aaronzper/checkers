use std::todo;
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
pub enum ActionResult {
    TookAction,
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
                        let highlight;
                        if board.state[x as usize][y as usize].piece == Some(self.side) {
                            highlight = true;
                            is_piece_this_side = true;
                        }
                        else {
                            highlight = false;
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
                    let piece = board.next_click().await.unwrap();
                    if board.state[piece.x as usize][piece.y as usize].piece != Some(self.side) {
                        continue;
                    }
                    else {
                        let valid_moves = board.valid_moves(&piece, self.side).unwrap();
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
                                continue;
                            }
                            else {
                                board.state[piece.x as usize][piece.y as usize].piece = None;
                                board.state[chosen_move.x as usize][chosen_move.y as usize].piece = Some(self.side);

                                let bigger_x;
                                let smaller_x;
                                let bigger_y;
                                let smaller_y;
                                if piece.x > chosen_move.x {
                                    bigger_x = piece.x;
                                    smaller_x = chosen_move.x;
                                }
                                else {
                                    bigger_x = chosen_move.x;
                                    smaller_x = piece.x;
                                }
                                if piece.y > chosen_move.y {
                                    bigger_y = piece.y;
                                    smaller_y = chosen_move.y;
                                }   
                                else {
                                    bigger_y = chosen_move.y;
                                    smaller_y = piece.y;
                                }

                                // Remove enemy pieces
                                for x in smaller_x..bigger_x {
                                    for y in smaller_y..bigger_y {
                                        if piece.x.abs_diff(chosen_move.x as u8) == piece.y.abs_diff(chosen_move.y as u8) {
                                            let other_team = match self.side {
                                                Side::Red => Side::Blue,
                                                Side::Blue => Side::Red
                                            };
                                            if board.state[x as usize][y as usize].piece == Some(other_team) {
                                                board.state[x as usize][y as usize].piece = None;
                                            }
                                        }
                                    }
                                }

                                board.draw().await.unwrap();
                                break;
                            }
                        }

                        break;
                    }
                }

                return ActionResult::TookAction;
            },
            _ => todo!()
        }
    }
}
