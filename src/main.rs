use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal, ExecutableCommand, QueueableCommand as _};
use std::io::{self, Result};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

enum InterfaceResult {
    None,
    Terminate,
}

trait Interface {
    fn handle_keypress(&mut self, event: KeyEvent) -> Result<InterfaceResult>;
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<InterfaceResult>;
    fn render(&self) -> Result<()>;
}

struct Window {
    width: u16,
    height: u16,
    horizontal_separator: String,
    prompt: String,
    history: Vec<String>,
}

impl Window {
    fn new(width: u16, height: u16) -> Self {
        let mut horizontal_separator = "-".repeat(width as usize);
        horizontal_separator.push_str("\r\n");

        let prompt = String::new();
        let history = Vec::new();

        Self {
            width,
            height,
            horizontal_separator,
            prompt,
            history,
        }
    }
}

impl Interface for Window {
    fn handle_keypress(&mut self, event: KeyEvent) -> Result<InterfaceResult> {
        match event.code {
            KeyCode::Esc => {
                self.width;
                return Ok(InterfaceResult::Terminate);
            }
            KeyCode::Backspace => {
                self.prompt.pop();
            }
            KeyCode::Char(c) => {
                if c == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(InterfaceResult::Terminate);
                } else {
                    self.prompt.push(c);
                }
            }
            KeyCode::Enter => {
                let old_prompt = std::mem::replace(&mut self.prompt, String::new());
                self.history.push(old_prompt);
            }
            code => {
                self.prompt = format!("Unhandled keycode: {:?}", code);
            }
        }
        Ok(InterfaceResult::None)
    }

    fn handle_resize(&mut self, width: u16, height: u16) -> Result<InterfaceResult> {
        self.width = width;
        self.height = height;
        Ok(InterfaceResult::None)
    }

    fn render(&self) -> Result<()> {
        let mut stdout = stdout();
        stdout.queue(Clear(ClearType::All))?;
        let n = self.history.len() as u16;
        let history_height = self.height - 2;
        let first_row = history_height - n.min(history_height);
        stdout.queue(MoveTo(0, first_row))?;
        self.history
            .iter()
            .skip(n.checked_sub(history_height).unwrap_or(0) as usize)
            .try_for_each(|msg| {
                stdout.queue(Print(msg))?;
                stdout.queue(Print("\r\n"))?;
                Ok::<(), std::io::Error>(())
            })?;
        stdout.queue(Print(&self.horizontal_separator))?;
        stdout.queue(Print(&self.prompt))?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    stdout.queue(EnterAlternateScreen)?;
    stdout.queue(Hide)?;
    stdout.flush()?;

    // down by {n}: \x1b[{n}B
    // up by {n}: \x1b[{n}A
    // right by {n}: \x1b[{n}C
    // left by {n}: \x1b[{n}D

    // specific row {n}: \x1b[{n};H
    // specific column {n}: \x1b[{n}G
    // specific row {n} and column {m}: \x1b[{n};{m}H

    let (width, height) = terminal::size()?;
    let mut window = Window::new(width, height);
    'outer: loop {
        while event::poll(std::time::Duration::ZERO)? {
            match event::read()? {
                Event::Key { 0: key_event } => {
                    let result = window.handle_keypress(key_event)?;
                    match result {
                        InterfaceResult::Terminate => {
                            break 'outer;
                        }
                        InterfaceResult::None => {}
                    }
                }
                Event::Resize {
                    0: width,
                    1: height,
                } => {
                    let result = window.handle_resize(width, height)?;
                    match result {
                        InterfaceResult::Terminate => {
                            break 'outer;
                        }
                        InterfaceResult::None => {}
                    }
                }
                e => {
                    window.prompt = format!("unhandled event: {:?}", e);
                }
            }
        }
        window.render()?;
        stdout.flush()?;
        thread::sleep(Duration::from_millis(16));
    }

    stdout.queue(LeaveAlternateScreen)?;
    stdout.queue(Show)?;
    stdout.flush()?;
    terminal::disable_raw_mode().unwrap();

    Ok(())
}
