use std::io::{Stdout, Write};

use crossterm::{
    Result,
    QueueableCommand,
    style::{Color, SetForegroundColor},
    style::Print,
    terminal::{EnterAlternateScreen, Clear, SetSize},
    cursor::MoveTo,
    style::SetBackgroundColor
};

pub enum BoardState {
    NoPiece,
    RedPiece,
    BluePiece
}

pub struct Board {
    pub width: u8,
    pub height: u8,
    pub terminal: Stdout,
    pub state: Vec<Vec<BoardState>>
}

impl Board {
    pub fn draw(&mut self) -> Result<()> {
        //self.terminal.queue(EnterAlternateScreen)?;
        self.terminal.queue(Clear(crossterm::terminal::ClearType::All))?;
        self.terminal.queue(SetSize(self.width as u16, self.height as u16))?;

        let mut x: u8 = 0;
        while x < self.width {
            self.state.push(Vec::default());

            let mut y: u8 = 0;
            while y < self.height {
                self.state[x as usize].push(BoardState::NoPiece);
                self.terminal.queue(MoveTo(x as u16 * 2, y as u16))?;

                let bg_color: Color;
                if (x % 2) == (y % 2) {
                    bg_color = Color::Black;
                    if y <= 2 {
                        self.state[x as usize][y as usize] = BoardState::RedPiece;
                    }
                    else if y >= 5 {
                        self.state[x as usize][y as usize] = BoardState::BluePiece;
                    }
                }
                else {
                    bg_color = Color::White;
                }

                self.terminal.queue(SetBackgroundColor(bg_color))?;

                let print_str = match self.state[x as usize][y as usize] {
                    BoardState::NoPiece => "  ",
                    BoardState::RedPiece => {
                        self.terminal.queue(SetForegroundColor(Color::Red))?;
                        "⦿ "
                    },
                    BoardState::BluePiece => {
                        self.terminal.queue(SetForegroundColor(Color::Blue))?;
                        "⦿ "
                    },

                };
                self.terminal.queue(Print(print_str))?;
                
                y += 1;
            }
            x += 1;
        }

        self.terminal.flush()?;

        Ok(())
    }
}
