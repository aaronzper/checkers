use std::{io::{Stdout, Write}, process::exit, sync::Arc};
use crate::actor::Actor;
use tokio::sync::Mutex;

use crossterm::{
    Result,
    QueueableCommand,
    style::{Color, SetForegroundColor},
    style::Print,
    terminal::{EnterAlternateScreen, Clear, SetSize, enable_raw_mode, disable_raw_mode, is_raw_mode_enabled},
    cursor::{MoveTo, Hide},
    style::SetBackgroundColor,
    event::{Event, KeyCode, MouseEventKind, EnableMouseCapture}, ExecutableCommand
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
    pub highlights: Vec<(u8, u8)>
}

impl Board {
    pub async fn new(width: u8, height: u8, terminal: Option<Stdout>) -> Result<Arc<Mutex<Board>>> {
        let board = Arc::new(Mutex::new(Board {
            width,
            height,
            terminal,
            state: Vec::default(),
            highlights: Vec::default()
        }));

        let mut board_lock = board.lock().await;

        match board_lock.terminal {
            None => (),
            Some(ref mut t) => {
                if is_raw_mode_enabled()? {
                    panic!("Game already exists using this terminal");
                }

                t.queue(EnterAlternateScreen)?;
                t.queue(EnableMouseCapture)?;
                t.queue(Hide)?; // Hide cursor
                enable_raw_mode()?;
                tokio::task::spawn(event_loop(Arc::clone(&board)));
            }
        };

        let mut x: u8 = 0;
        while x < board_lock.width {
            board_lock.state.push(Vec::default());

            let mut y: u8 = 0;
            while y < board_lock.height {
                let piece;
                if (x % 2) == (y % 2) {
                    if y <= 2 {
                        piece = BoardState::RedPiece;
                    }
                    else if y >= (board_lock.height - 3) {
                        piece = BoardState::BluePiece;
                    }
                    else {
                        piece = BoardState::NoPiece;
                    }
                }
                else {
                    piece = BoardState::NoPiece;
                }

                board_lock.state[x as usize].push(piece);
                
                y += 1;
            }
            x += 1;
        }

        drop(board_lock);
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
                if self.highlights.contains(&(x, y)) {
                    bg_color = Color::DarkYellow;
                }
                else if (x % 2) == (y % 2) {
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

fn terminal_cord_to_board(column: u16, row: u16) -> (u8, u8) {
    ((column / 2) as u8, row as u8)
}

async fn event_loop(board: Arc<Mutex<Board>>) {
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(event) => {
                if event.code == KeyCode::Esc {
                    // TODO: Show cursor, reset terminal, etc
                    exit(0);
                }
            },
            Event::Mouse(event) => {
                if event.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    let mut board_lock = board.lock().await;
                    board_lock.highlights.push(terminal_cord_to_board(event.column, event.row));
                    board_lock.draw().await.unwrap();
                }
            },
            _ => continue
        }
    }

}
