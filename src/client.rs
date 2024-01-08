use anyhow::Result;
use std::{
    io::{stdout, ErrorKind, Read, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};

use crate::{
    constants::MAX_MESSAGE_SIZE,
    window::{Screen, Window, WindowAction},
};

pub struct Client {
    remote_address: String,
}

impl Client {
    pub fn new(remote_address: String) -> Self {
        Self { remote_address }
    }

    pub fn run(&self) -> Result<()> {
        let mut stdout = stdout();

        // Setup terminal
        terminal::enable_raw_mode()?;
        stdout.queue(EnterAlternateScreen)?;
        stdout.queue(Print("Connecting..."))?;
        stdout.flush()?;

        // initiate connection
        let mut stream = TcpStream::connect(&self.remote_address)?;
        let mut buffer = [0u8; MAX_MESSAGE_SIZE];
        stream.set_nonblocking(true)?;

        let (sender, receiver) = mpsc::channel::<String>();

        let (width, height) = terminal::size()?;
        let mut window = Screen::new(width, height, sender);
        'outer: loop {
            // receive from server
            loop {
                let n = match stream.read(&mut buffer) {
                    Ok(n) => Ok(n),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) => Err(e),
                }?;
                let message = String::from_utf8(buffer[..n].to_vec())?;
                window.add_message(message);
            }

            loop {
                match receiver.recv_timeout(Duration::ZERO) {
                    Ok(message) => {
                        stream.write_all(&message.as_bytes()).unwrap();
                    }
                    Err(_) => break,
                }
            }

            // handle input
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

            //render
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
