use std::{sync::{Arc, atomic::AtomicBool}, io::{Stdout, Write}};
use tokio::{sync::mpsc::{channel, Receiver, Sender}, task::JoinHandle};
use crate::point::Point;
use crate::game::board::Board;
use crate::Side;
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

pub struct TerminalWrapper {
    pub terminal: Stdout,
    pub exit_requested: Arc<AtomicBool>,
    click_events_rx: Receiver<Point>,
    event_loop_handle: JoinHandle<()>
}

impl Drop for TerminalWrapper {
    fn drop(&mut self) {
        self.event_loop_handle.abort();
        self.terminal.queue(Show).unwrap(); // Show cursor. Unwrap instead of ? since we're in a drop
        self.terminal.queue(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }
}

impl TerminalWrapper {
    pub fn new(mut terminal: Stdout) -> Result<TerminalWrapper> {
        let (tx, rx) = channel(8);
        let exit_requested = Arc::new(AtomicBool::new(false));

        if is_raw_mode_enabled()? {
            panic!("Game already exists using this terminal");
        }

        terminal.queue(EnterAlternateScreen)?;
        terminal.queue(Hide)?; // Hide cursor
        enable_raw_mode()?;

        let event_loop_handle = tokio::task::spawn(event_loop(Arc::clone(&exit_requested), tx));

        let wrapper = TerminalWrapper {
            terminal,
            click_events_rx: rx,
            exit_requested,
            event_loop_handle
        };

        Ok(wrapper)
    }

    pub async fn draw(&mut self, board: &Board) -> Result<()> {
        self.terminal.queue(Clear(crossterm::terminal::ClearType::All))?;
        self.terminal.queue(SetSize(board.width as u16, board.height as u16))?;

        for x in 0..board.width {
            for y in 0..board.height {
                self.terminal.queue(MoveTo(x as u16 * 2, y as u16))?;

                let bg_color: Color;
                if board.state[x as usize][y as usize].highlighted {
                    bg_color = Color::DarkYellow;
                }
                else if (x % 2) == (y % 2) {
                    bg_color = Color::Black;
                }
                else {
                    bg_color = Color::White;
                }

                self.terminal.queue(SetBackgroundColor(bg_color))?;

                let mut print_str: String;
                match board.state[x as usize][y as usize].piece {
                    None => print_str = "  ".to_string(),
                    Some(ref p) => {
                        print_str = "⦿".to_string();

                        match p.side {
                            Side::Red => self.terminal.queue(SetForegroundColor(Color::Red))?,
                            Side::Blue => self.terminal.queue(SetForegroundColor(Color::Blue))?,
                        };
                        
                        if p.crowned {
                            print_str += "♕";
                        }
                        else {
                            print_str += " ";
                        }
                    },
                };
                self.terminal.queue(Print(print_str))?;
            }
        }

        self.terminal.queue(SetBackgroundColor(Color::Reset))?;
        self.terminal.queue(SetForegroundColor(Color::Reset))?;
        self.terminal.queue(Print("\n\r"))?;
        self.terminal.flush()?;

        Ok(())
    }

    pub async fn next_click(&mut self, board: &Board) -> Result<Point> {
        self.terminal.execute(EnableMouseCapture)?;
        
        loop {
            match self.click_events_rx.recv().await {
                Some(click) => {
                    if click.x < board.width && click.y < board.height {
                        self.terminal.execute(DisableMouseCapture)?;
                        return Ok(click);
                    }
                },
                None => {
                    self.terminal.execute(DisableMouseCapture)?;
                    return Err(ErrorKind::new(std::io::ErrorKind::Other, "Click Event Channel Error"));
                }
            }
        }
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
