use crossterm::Result;
use crate::{piece::Piece, side::Side, point::Point, actor::Action};
use std::collections::HashMap;

#[derive(Clone)]
pub struct BoardState {
    pub piece: Option<Piece>,
    pub highlighted: bool
}

#[derive(Clone)]
pub struct Board {
    pub width: u8,
    pub height: u8,
    pub state: Vec<Vec<BoardState>>
}

impl Board {
    pub fn new(width: u8, height: u8) -> Result<Board> {
        let mut board = Board {
            width,
            height,
            state: Vec::default(),
        };

        let mut x: u8 = 0;
        while x < board.width {
            board.state.push(Vec::default());

            let mut y: u8 = 0;
            while y < board.height {
                let piece;
                if (x % 2) == (y % 2) {
                    if y <= 2 {
                        piece = Some(Piece { side: Side::Red, crowned: false});
                    }
                    else if y >= (board.height - 3) {
                        piece = Some(Piece { side: Side::Blue, crowned: false});
                    }
                    else {
                        piece = None;
                    }
                }
                else {
                    piece = None
                }

                board.state[x as usize].push(BoardState {
                    piece,
                    highlighted: false
                });
                
                y += 1;
            }
            x += 1;
        }

        Ok(board)
    }

    pub fn valid_moves(&self, acting_piece: &Point, piece: Piece) -> Result<Vec<Point>> {
        let mut actions = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                actions.push(Point { x, y });
            }
        }

        let actions_filtered: Vec<Point> = actions.into_iter().filter(|point| {
            let x = point.x as usize;
            let y = point.y as usize;

            if acting_piece.x.abs_diff(x as u8) != acting_piece.y.abs_diff(y as u8) { // Non-diagonal spaces aren't valid moves
                return false;
            }

            if !piece.crowned { // Don't let pieces go backwards, but ignore if crowned
                if piece.side == Side::Red { 
                    if acting_piece.y > y as u8 { 
                        return false;
                    }
                }
                else if piece.side == Side::Blue { // Which direction is "backwards" depends on the side
                    if acting_piece.y < y as u8 {
                        return false;
                    }
                }
            }
        
            if self.state[x][y].piece.is_some() { // Spaces with pieces aren't valid moves
                return false;
            }

            return true;
        }).collect();

        Ok(actions_filtered)
    }

    pub fn get_all_moves(&self, side: Side) -> HashMap<Point, Vec<Point>> {
        let mut all_moves = HashMap::new();
        for x in 0..self.width {
            for y in 0..self.height {
                let piece = self.state[x as usize][y as usize].piece;
                if side.piece_is_friendly(&piece) {
                    let moves = self.valid_moves(&Point {x, y}, piece.unwrap()).unwrap();
                    if moves.len() != 0 {
                        all_moves.insert(Point { x, y }, moves);
                    }
                }
            }
        }

        all_moves
    }

    pub async fn do_action(&mut self, action: &Action) {
        let from_piece = self.state[action.from.x as usize][action.from.y as usize].piece.unwrap();

        // Crown the piece, if applicable
        let crowned = if from_piece.crowned {
            true // If it's already crowned, keep it that way
        }
        else {
            if (from_piece.side == Side::Red && action.to.y == self.height - 1) || (from_piece.side == Side::Blue && action.to.y == 0) {
                true
            }
            else {
                false
            }
        };

        // Actually move the piece
        self.state[action.from.x as usize][action.from.y as usize].piece = None;
        self.state[action.to.x as usize][action.to.y as usize].piece = Some(Piece { side: from_piece.side, crowned });

        // Find out the bigger/smaller x and y from the source/destination for below
        let bigger_x;
        let smaller_x;
        let bigger_y;
        let smaller_y;
        if action.from.x > action.to.x {
            bigger_x = action.from.x;
            smaller_x = action.to.x;
        }
        else {
            bigger_x = action.to.x;
            smaller_x = action.from.x;
        }
        if action.from.y > action.to.y {
            bigger_y = action.from.y;
            smaller_y = action.to.y;
        }   
        else {
            bigger_y = action.to.y;
            smaller_y = action.from.y;
        }

        // Remove enemy pieces between source and destination
        for x in smaller_x..bigger_x {
            for y in smaller_y..bigger_y {
                if action.from.x.abs_diff(x as u8) == action.from.y.abs_diff(y as u8) {
                    if from_piece.side.piece_is_hostile(&self.state[x as usize][y as usize].piece) {
                        self.state[x as usize][y as usize].piece = None;
                    }
                }
            }
        }
    }
}
