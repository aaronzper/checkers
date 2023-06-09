use crate::piece::Piece;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
    Red,
    Blue
}

impl Side {
    pub fn piece_is_friendly(&self, piece: &Option<Piece>) -> bool {
        match piece {
            None => false,
            Some(x) => {
                x.side == *self
            }
        }
    }

    pub fn piece_is_hostile(&self, piece: &Option<Piece>) -> bool {
        match piece {
            None => false,
            Some(x) => {
                x.side != *self
            }
        }
    }

}

