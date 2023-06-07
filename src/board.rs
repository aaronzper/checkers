use std::{io::{Stdout, Write}, sync::{Arc, atomic::AtomicBool}};
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

use crate::{point::Point, actor::{ActorType, Actor, ActionResult, Action}};

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone)]
pub struct Piece {
    pub side: Side,
    pub crowned: bool
}

pub struct BoardState {
    pub piece: Option<Piece>,
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

                let mut print_str: String;
                match self.state[x as usize][y as usize].piece {
                    None => print_str = "  ".to_string(),
                    Some(ref p) => {
                        print_str = "⦿".to_string();

                        match p.side {
                            Side::Red => terminal.queue(SetForegroundColor(Color::Red))?,
                            Side::Blue => terminal.queue(SetForegroundColor(Color::Blue))?,
                        };
                        
                        if p.crowned {
                            print_str += "♕";
                        }
                        else {
                            print_str += " ";
                        }
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

    pub async fn play(&mut self, red_actor_type: ActorType, blue_actor_type: ActorType) -> Result<Option<Side>> {
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

        let mut winner = None;
        // TODO: Cancel out if too many AI-on-AI iterations without kill
        loop {
            if self.exit_requested.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(winner);
            }
            
            if winner.is_none() {
                match red_actor.act(self).await {
                    ActionResult::NoPiecesLeft => {
                        println!("Blue won!");
                        winner = Some(Side::Blue);
                        continue;
                    },
                    ActionResult::TookAction(action) => {
                        self.do_action(&action).await;
                    }
                }

                match blue_actor.act(self).await {
                    ActionResult::NoPiecesLeft => {
                        println!("Red won!");
                        winner = Some(Side::Red);
                        continue;
                    },
                    ActionResult::TookAction(action) => {
                        self.do_action(&action).await;
                    }
                }

            }
        }
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

        self.draw().await.unwrap();
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
