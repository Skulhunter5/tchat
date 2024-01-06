use anyhow::Result;
use crossterm::event::Event;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal, QueueableCommand as _};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;
use window::{Screen, Window, WindowAction};

mod util;
mod window;

fn main() -> Result<()> {
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
