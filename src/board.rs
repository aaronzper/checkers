use std::{io::{Stdout, Write}, sync::{Arc, atomic::AtomicBool}, thread::sleep_ms};
use tokio::sync::mpsc::{Sender, Receiver, channel};
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

use crate::{point::Point, actor::{ActorType, Actor, ActionResult}};

#[derive(Copy, Clone, PartialEq)]
pub enum Side {
    Red,
    Blue
}

pub struct BoardState {
    pub piece: Option<Side>,
    pub highlighted: bool
}

pub struct Board {
    pub width: u8,
    pub height: u8,
    pub terminal: Option<Stdout>,
    pub state: Vec<Vec<BoardState>>,
    exit_requested: Arc<AtomicBool>,
    click_events_rx: Receiver<Point>
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
        terminal.queue(Print("\n\r"))?;
        terminal.flush()?;

        Ok(())
    }

    pub async fn next_click(&mut self) -> Result<Point> {
        let mut t = self.terminal.as_ref().expect("Ran next_click on board without terminal");
        t.execute(EnableMouseCapture)?;
        
        loop {
            match self.click_events_rx.recv().await {
                Some(click) => {
                    if click.x < self.width && click.y < self.height {
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

    pub async fn play(&mut self, red_actor_type: ActorType, blue_actor_type: ActorType) -> Result<()> {
        if self.terminal.is_none() && (red_actor_type == ActorType::Human || blue_actor_type == ActorType::Human) {
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

        let mut game_over = false;
        loop {
            if self.exit_requested.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(());
            }
            
            if !game_over {
                if red_actor.act(self).await == ActionResult::NoPiecesLeft {
                    println!("Blue won!");
                    game_over = true;
                }
                else if blue_actor.act(self).await == ActionResult::NoPiecesLeft {
                    println!("Red won!");
                    game_over = true;
                }
            }
        }
    }

    pub fn valid_moves(&self, acting_piece: &Point, side: Side) -> Result<Vec<Point>> {
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

            if side == Side::Red { // Can't go backwards | TODO: Add crowning functionality
                if acting_piece.y > y as u8 { 
                    return false;
                }
            }
            else if side == Side::Blue { // Which direction is "backwards" depends on the side
                if acting_piece.y < y as u8 {
                    return false;
                }
            }
        
            if self.state[x][y].piece.is_some() { // Spaces with pieces aren't valid moves
                return false;
            }

            return true;
        }).collect();

        Ok(actions_filtered)
    }
}

fn terminal_cord_to_board(column: u16, row: u16) -> Point {
    ((column / 2) as u8, row as u8).into()
}

async fn event_loop(exit_requested: Arc<AtomicBool>, click_events_tx: Sender<Point>) {
    loop {
        match crossterm::event::read().unwrap() {
            Event::Key(event) => {
                if event.code == KeyCode::Esc {
                    exit_requested.store(true, std::sync::atomic::Ordering::Relaxed);
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
