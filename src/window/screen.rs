use std::io::stdout;

use crate::util::Rectangle;
use crate::window::{Window, WindowAction};
use anyhow::Result;
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
    pub fn new(width: u16, height: u16) -> Self {
        let chat = Chat::new(Rectangle::new(0, 0, width, height));

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
