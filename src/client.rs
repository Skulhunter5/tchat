use anyhow::Result;
use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};

use crate::window::{Screen, Window, WindowAction};

pub struct Client;

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) -> Result<()> {
        let mut stdout = stdout();

        terminal::enable_raw_mode()?;
        stdout.queue(EnterAlternateScreen)?;
        stdout.flush()?;

        let (width, height) = terminal::size()?;
        let mut window = Screen::new(width, height);
        'outer: loop {
            while event::poll(std::time::Duration::ZERO)? {
                match event::read()? {
                    Event::Key { 0: key_event } => match window.handle_keypress(key_event)? {
                        WindowAction::Terminate => {
                            break 'outer;
                        }
                        WindowAction::None => {}
                    },
                    Event::Resize {
                        0: width,
                        1: height,
                    } => match window.handle_resize(width, height)? {
                        WindowAction::Terminate => {
                            break 'outer;
                        }
                        WindowAction::None => {}
                    },
                    e => {
                        window.set_prompt(format!("unhandled event: {:?}", e));
                    }
                }
            }
            window.render()?;
            stdout.flush()?;
            thread::sleep(Duration::from_millis(16));
        }

        stdout.queue(LeaveAlternateScreen)?;
        stdout.flush()?;
        terminal::disable_raw_mode().unwrap();

        Ok(())
    }
}
