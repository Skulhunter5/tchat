use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use std::{
    io::{stdout, ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    style::Print,
    terminal::{self, Clear, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};

use crate::{
    constants::MAX_MESSAGE_SIZE,
    window::{Screen, Window, WindowAction},
};

static TERMINATE: AtomicBool = AtomicBool::new(false);

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

        let (messages_out_sender, messages_out_receiver) = crossbeam_channel::unbounded::<String>();
        let (messages_in_sender, messages_in_receiver) = crossbeam_channel::unbounded::<String>();

        let messages_in = messages_in_sender;
        let messages_out = messages_out_receiver;

        // initialize connection
        let mut stream = TcpStream::connect(&self.remote_address)?;
        let mut buffer = [0u8; MAX_MESSAGE_SIZE];
        stream.set_nonblocking(true)?;

        let (width, height) = terminal::size()?;
        let mut window = Screen::new(width, height, messages_in_receiver, messages_out_sender);
        'outer: while !TERMINATE.load(Ordering::Relaxed) {
            loop {
                let n = match stream.read(&mut buffer) {
                    Ok(n) => Ok(n),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                    Err(e) => Err(e),
                }?;
                if n == 0 {
                    continue;
                }
                let message = String::from_utf8(buffer[..n].to_vec())?;
                messages_in.send(message)?;
            }

            while let Ok(message) = messages_out.try_recv() {
                stream.write_all(message.as_bytes())?;
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

            window.update()?;

            //render
            stdout.queue(Clear(terminal::ClearType::All))?;
            stdout.flush()?;
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
