use std::todo;

use crate::board::BoardState;

#[derive(Clone, Copy)]
pub enum RecursiveActorType {
    Random,
    MostKills
}

#[derive(Clone, Copy)]
pub enum ActorType {
    Human,
    Random,
    MostKills,
    Recursive(RecursiveActorType)
}

#[derive(Clone, Copy)]
pub struct Actor {
    pub actor_type: ActorType
}

impl Actor {
    pub fn act(&self, state: &mut Vec<Vec<BoardState>>) {
        todo!()
    }
}
