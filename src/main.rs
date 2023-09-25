use std::fs::File;
use std::io;
use std::io::{Read, Write};
use crossterm::{event, ExecutableCommand, terminal};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{Clear, ClearType};

fn main() -> io::Result<()> {
    let mut stdin = io::stdin();
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
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key { 0: key_event } => {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char(c) => {
                            stdout.execute(Clear(ClearType::CurrentLine)).unwrap();
                            print!("Pressed: {}\r", c);
                            stdout.flush()?;
                        }
                        code => {
                            print!("Pressed code: {:?}\r", code);
                            stdout.flush()?;
                        }
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
