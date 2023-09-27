use std::fs::File;
use std::io::{self, Result};
use std::io::{Read, Write};
use crossterm::{event, ExecutableCommand, terminal};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{Clear, ClearType};

enum InterfaceResult {
    None,
    Terminate,
}

trait Interface {
    fn handle_keypress(&self, event: KeyEvent) -> Result<InterfaceResult>;
}

struct Window {
    width: u16,
    height: u16,
}

impl Window {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
        }
    }
}

impl Interface for Window {
    fn handle_keypress(&self, event: KeyEvent) -> Result<InterfaceResult> {
        let mut stdout = io::stdout();

        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                return Ok(InterfaceResult::Terminate);
            }
            KeyCode::Char(c) => {
                stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                print!("Pressed: {}\r", c);
                stdout.flush()?;
            }
            code => {
                stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                print!("Pressed code: {:?}\r", code);
                stdout.flush()?;
            }
        }
        Ok(InterfaceResult::None)
    }
}

fn jump_to(x: u16, y: u16) -> Result<()> {
    print!("\x1b[{};{}H", y, x);
    io::stdout().flush()
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode().unwrap();
    stdout.execute(Hide).unwrap();
    stdout.execute(Clear(ClearType::All)).unwrap();

    // down by {n}: \x1b[{n}B
    // up by {n}: \x1b[{n}A
    // right by {n}: \x1b[{n}C
    // left by {n}: \x1b[{n}D

    // specific row {n}: \x1b[{n};H
    // specific column {n}: \x1b[{n}G
    // specific row {n} and column {m}: \x1b[{n};{m}H

    let (width, height) = terminal::size()?;
    let window = Window::new(width, height);
    jump_to(window.width-1, window.height-1)?;
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key { 0: key_event } => {
                    let result = window.handle_keypress(key_event);
                    match result? {
                        InterfaceResult::Terminate => {
                            break;
                        }
                        InterfaceResult::None => {}
                    }
                }
                Event::Resize { 0: width, 1: height } => {
                    println!("New Size: {}x{}\r", width, height);
                }
                e => {
                    println!("Event: {:?}", e);
                }
            }
        }
    }

    //stdout.execute(Clear(ClearType::All))?;
    print!("\x1b[2J");
    stdout.flush()?;
    terminal::disable_raw_mode().unwrap();
    stdout.execute(Show).unwrap();
    println!();

    Ok(())
}
