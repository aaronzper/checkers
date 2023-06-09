use crate::side::Side;

#[derive(Copy, Clone)]
pub struct Piece {
    pub side: Side,
    pub crowned: bool
}
