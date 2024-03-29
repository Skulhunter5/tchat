use std::io::stdout;

use crate::util::Rectangle;
use crate::window::{Window, WindowAction};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use crossterm::event::KeyEvent;
use crossterm::terminal::{Clear, ClearType};
use crossterm::QueueableCommand as _;

use super::Chat;

pub struct Screen {
    width: u16,
    height: u16,
    chat: Chat,
}

impl Screen {
    pub fn new(
        width: u16,
        height: u16,
        messages_in: Receiver<String>,
        messages_out: Sender<String>,
    ) -> Self {
        let chat = Chat::new(
            Rectangle::new(0, 0, width, height),
            messages_in,
            messages_out,
        );

        Self {
            width,
            height,
            chat,
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.chat.set_prompt(prompt);
    }
}

impl Window for Screen {
    fn update(&mut self) -> Result<()> {
        self.chat.update()?;
        Ok(())
    }

    fn handle_keypress(&mut self, event: KeyEvent) -> Result<WindowAction> {
        self.chat.handle_keypress(event)
    }

    fn handle_resize(&mut self, width: u16, height: u16) -> Result<WindowAction> {
        self.width = width;
        self.height = height;

        // resize chat
        self.chat.resize(Rectangle::new(0, 0, width, height));

        Ok(WindowAction::None)
    }

    fn render(&self) -> Result<()> {
        stdout().queue(Clear(ClearType::All))?;
        self.chat.render()
    }
}
