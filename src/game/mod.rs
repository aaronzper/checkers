mod terminal;
mod board;

use std::io::Stdout;
use crossterm::{Result, ErrorKind};
use crate::{actor::{ActorType, Actor, ActionResult}, side::Side, piece::Piece};
use self::{terminal::TerminalWrapper, board::Board};

pub struct GameResult {
    pub moves: usize,
    pub winner: Side
}

pub struct Game {
    pub board: Board,
    pub terminal_wrapper: Option<TerminalWrapper>,
}

impl Game {
    pub fn new(width: u8, height: u8, terminal: Option<Stdout>) -> Result<Game> {
        let terminal_wrapper = match terminal {
            Some(t) => Some(TerminalWrapper::new(t)?),
            None => None
        };

        let board = Board::new(width, height)?;

        Ok(Game {
            board,
            terminal_wrapper
        })
    }

    pub async fn play(&mut self, red_actor_type: ActorType, blue_actor_type: ActorType) -> Result<Option<GameResult>> {
        if self.terminal_wrapper.is_none() && (red_actor_type == ActorType::Human || blue_actor_type == ActorType::Human) {
            return Err(ErrorKind::new(std::io::ErrorKind::Unsupported, "Cannot have human actor on virtual board"));
        }

        let red_actor = Actor {
            actor_type: red_actor_type,
            side: Side::Red
        };
        let blue_actor = Actor {
            actor_type: blue_actor_type,
            side: Side::Blue
        };

        let mut winner = None;
        // TODO: Cancel out if too many AI-on-AI iterations without kill
        loop {
            // Can ignore exit request if no terminal
            if self.terminal_wrapper.is_some() {
                if self.terminal_wrapper.as_ref().unwrap().exit_requested.load(std::sync::atomic::Ordering::Relaxed) {
                    return Ok(winner);
                }
            }
            else if self.terminal_wrapper.is_none() && winner.is_some() {
                return Ok(winner);
            }
            
            let mut moves = 0;
            if winner.is_none() {
                match red_actor.act(self).await {
                    ActionResult::NoPiecesLeft => {
                        if self.terminal_wrapper.is_some() {
                            println!("Blue won!");
                        }
                        winner = Some(GameResult { moves, winner: Side::Blue });
                        continue;
                    },
                    ActionResult::TookAction(action) => {
                        self.board.do_action(&action).await;
                        if self.terminal_wrapper.is_some() {
                            self.terminal_wrapper.as_mut().unwrap().draw(&self.board).await?;
                        }
                        moves += 1;
                    }
                }

                match blue_actor.act(self).await {
                    ActionResult::NoPiecesLeft => {
                        if self.terminal_wrapper.is_some() {
                            println!("Red won!");
                        }
                        winner = Some(GameResult { moves, winner: Side::Red });
                        continue;
                    },
                    ActionResult::TookAction(action) => {
                        self.board.do_action(&action).await;
                        if self.terminal_wrapper.is_some() {
                            self.terminal_wrapper.as_mut().unwrap().draw(&self.board).await?;
                        }
                        moves += 1;
                    }
                }

            }
        }
    }
}
