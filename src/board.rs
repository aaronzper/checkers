use std::{io::{Stdout, Write}, process::exit};
use crate::actor::Actor;
use futures::StreamExt;

use crossterm::{
    Result,
    QueueableCommand,
    style::{Color, SetForegroundColor},
    style::Print,
    terminal::{EnterAlternateScreen, Clear, SetSize, enable_raw_mode, disable_raw_mode, is_raw_mode_enabled},
    cursor::MoveTo,
    style::SetBackgroundColor,
    event::{Event, KeyCode, EventStream}
};

pub enum BoardState {
    NoPiece,
    RedPiece,
    BluePiece
}

pub struct Board {
    pub width: u8,
    pub height: u8,
    pub terminal: Option<Stdout>,
    pub state: Vec<Vec<BoardState>>,
    pub actors: (Actor, Actor)
}

impl Board {
    pub fn new(width: u8, height: u8, terminal: Option<Stdout>, actors: (Actor, Actor)) -> Result<Board> {
        let mut board = Board {
            width,
            height,
            terminal,
            state: Vec::default(),
            actors
        };

        match board.terminal {
            None => (),
            Some(ref mut t) => {
                if is_raw_mode_enabled()? {
                    panic!("Game already exists using this terminal");
                }

                t.queue(EnterAlternateScreen)?;
                enable_raw_mode()?;
                tokio::task::spawn(event_loop());
            }
        };

        let mut x: u8 = 0;
        while x < board.width {
            board.state.push(Vec::default());

            let mut y: u8 = 0;
            while y < board.height {
                let piece;
                if (x % 2) == (y % 2) {
                    if y <= 2 {
                        piece = BoardState::RedPiece;
                    }
                    else if y >= (board.height - 3) {
                        piece = BoardState::BluePiece;
                    }
                    else {
                        piece = BoardState::NoPiece;
                    }
                }
                else {
                    piece = BoardState::NoPiece;
                }

                board.state[x as usize].push(piece);
                
                y += 1;
            }
            x += 1;
        }

        Ok(board)
    }

    pub async fn draw(&mut self) -> Result<()> {
        let mut terminal = self.terminal.as_ref().unwrap();
        terminal.queue(Clear(crossterm::terminal::ClearType::All))?;
        terminal.queue(SetSize(self.width as u16, self.height as u16))?;

        let mut x: u8 = 0;
        while x < self.width {
            let mut y: u8 = 0;
            while y < self.height {
                terminal.queue(MoveTo(x as u16 * 2, y as u16))?;

                let bg_color: Color;
                if (x % 2) == (y % 2) {
                    bg_color = Color::Black;
                }
                else {
                    bg_color = Color::White;
                }

                terminal.queue(SetBackgroundColor(bg_color))?;

                let print_str = match self.state[x as usize][y as usize] {
                    BoardState::NoPiece => "  ",
                    BoardState::RedPiece => {
                        terminal.queue(SetForegroundColor(Color::Red))?;
                        "⦿ "
                    },
                    BoardState::BluePiece => {
                        terminal.queue(SetForegroundColor(Color::Blue))?;
                        "⦿ "
                    },

                };
                terminal.queue(Print(print_str))?;
                
                y += 1;
            }
            x += 1;
        }

        terminal.queue(SetBackgroundColor(Color::Reset))?;
        terminal.queue(SetForegroundColor(Color::Reset))?;
        terminal.flush()?;


        Ok(())

    }


}

async fn event_loop() {
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(event) => {
                if event.code == KeyCode::Esc {
                    exit(0);
                }
            },
            _ => continue
        }
    }

}
