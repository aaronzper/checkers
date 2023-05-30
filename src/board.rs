use std::{io::{Stdout, Write}, sync::{Arc, atomic::AtomicBool}, collections::HashMap};
use tokio::sync::mpsc::{Sender, Receiver, channel, error::TryRecvError};
use crossterm::{
    Result,
    QueueableCommand,
    ExecutableCommand,
    style::{Color, SetForegroundColor},
    style::Print,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, SetSize, enable_raw_mode, disable_raw_mode, is_raw_mode_enabled},
    cursor::{MoveTo, Hide, Show},
    style::SetBackgroundColor,
    event::{Event, KeyCode, MouseEventKind, EnableMouseCapture, DisableMouseCapture}, ErrorKind
};

pub enum Side {
    Red,
    Blue
}

pub struct BoardState {
    piece: Option<Side>,
    highlighted: bool
}

pub struct Board {
    pub width: u8,
    pub height: u8,
    pub terminal: Option<Stdout>,
    pub state: Vec<Vec<BoardState>>,
    exit_requested: Arc<AtomicBool>,
    click_events_rx: Receiver<(u8, u8)>
}

impl Drop for Board {
    fn drop(&mut self) {
        if let Some(ref mut t) = self.terminal {
            t.queue(Show).unwrap(); // Show cursor. Unwrap instead of ? since we're in a drop
            t.queue(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
        }
    }
}

impl Board {
    pub async fn new(width: u8, height: u8, terminal: Option<Stdout>) -> Result<Board> {
        let (tx, rx) = channel(8);

        let mut board = Board {
            width,
            height,
            terminal,
            state: Vec::default(),
            exit_requested: Arc::new(AtomicBool::new(false)),
            click_events_rx: rx,
        };

        if let Some(ref mut t) = board.terminal {
            if is_raw_mode_enabled()? {
                panic!("Game already exists using this terminal");
            }

            t.queue(EnterAlternateScreen)?;
            t.queue(Hide)?; // Hide cursor
            enable_raw_mode()?;
            tokio::task::spawn(event_loop(Arc::clone(&board.exit_requested), tx));
        };

        let mut x: u8 = 0;
        while x < board.width {
            board.state.push(Vec::default());

            let mut y: u8 = 0;
            while y < board.height {
                let piece;
                if (x % 2) == (y % 2) {
                    if y <= 2 {
                        piece = Some(Side::Red);
                    }
                    else if y >= (board.height - 3) {
                        piece = Some(Side::Blue);
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
                if self.state[x as usize][y as usize].highlighted {
                    bg_color = Color::DarkYellow;
                }
                else if (x % 2) == (y % 2) {
                    bg_color = Color::Black;
                }
                else {
                    bg_color = Color::White;
                }

                terminal.queue(SetBackgroundColor(bg_color))?;

                let print_str = match self.state[x as usize][y as usize].piece {
                    None => "  ",
                    Some(Side::Red) => {
                        terminal.queue(SetForegroundColor(Color::Red))?;
                        "⦿ "
                    },
                    Some(Side::Blue) => {
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

    async fn next_click(&mut self) -> Result<(u8, u8)> {
        let mut t = self.terminal.as_ref().expect("Ran next_click on board without terminal");
        t.execute(EnableMouseCapture)?;
        
        loop {
            match self.click_events_rx.recv().await {
                Some(click) => {
                    if click.0 < self.width && click.1 < self.height {
                        t.execute(DisableMouseCapture)?;
                        return Ok(click);
                    }
                },
                None => {
                    t.execute(DisableMouseCapture)?;
                    return Err(ErrorKind::new(std::io::ErrorKind::Other, "Click Event Channel Error"));
                }
            }
        }

    }

    pub async fn play(&mut self) -> Result<()> {
        loop {
            /*if self.exit_requested.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(());
            }*/

            let click = self.next_click().await.unwrap();
            self.state[click.0 as usize][click.1 as usize].highlighted = !self.state[click.0 as usize][click.1 as usize].highlighted;
            self.draw().await?;
        }
    }
}

fn terminal_cord_to_board(column: u16, row: u16) -> (u8, u8) {
    ((column / 2) as u8, row as u8)
}

async fn event_loop(exit_requested: Arc<AtomicBool>, click_events_tx: Sender<(u8, u8)>) {
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(event) => {
                if event.code == KeyCode::Esc {
                    //exit_requested.store(true, std::sync::atomic::Ordering::Relaxed);
                    return;
                }
            },
            Event::Mouse(event) => {
                if event.kind == MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    click_events_tx.send(terminal_cord_to_board(event.column, event.row)).await.unwrap_or(());
                }
            },
            _ => continue
        }
    }

}
