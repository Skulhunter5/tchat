use std::io::stdout;
use std::sync::mpsc::Sender;

use crate::util::Rectangle;

use super::Window;
use super::WindowAction;
use anyhow::Result;
use crossterm::cursor::MoveTo;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::style::Print;
use crossterm::QueueableCommand;

pub struct Chat {
    rect: Rectangle<u16>,
    horizontal_separator: String,
    prompt: String,
    history: Vec<String>,
    sender: Sender<String>,
}

impl Chat {
    pub fn new(rect: Rectangle<u16>, sender: Sender<String>) -> Self {
        let horizontal_separator = "-".repeat(rect.w as usize);

        let prompt = String::new();
        let history = Vec::new();

        Self {
            rect,
            prompt,
            horizontal_separator,
            history,
            sender,
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }

    pub fn resize(&mut self, rect: Rectangle<u16>) {
        self.rect = rect;
        self.horizontal_separator = "-".repeat(self.rect.w as usize);
    }

    pub fn add_message(&mut self, message: String) {
        self.history.push(message);
    }
}

impl Window for Chat {
    fn handle_keypress(&mut self, event: crossterm::event::KeyEvent) -> Result<WindowAction> {
        match event.code {
            KeyCode::Esc => {
                return Ok(WindowAction::Terminate);
            }
            KeyCode::Backspace => {
                self.prompt.pop();
            }
            KeyCode::Char(c) => {
                if c == 'c' && event.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(WindowAction::Terminate);
                } else {
                    self.prompt.push(c);
                }
            }
            KeyCode::Enter => {
                let message = std::mem::replace(&mut self.prompt, String::new());
                self.sender.send(message).unwrap();
                //self.history.push(old_prompt);
            }
            code => {
                self.prompt = format!("Unhandled keycode: {:?}", code);
            }
        }
        Ok(WindowAction::None)
    }

    fn handle_resize(&mut self, _width: u16, _height: u16) -> Result<WindowAction> {
        todo!()
    }

    fn render(&self) -> Result<()> {
        let mut stdout = stdout();

        let (x, y, width, height) = self.rect.unpack();

        let n = self.history.len() as u16;
        let history_height = height - 2;
        let first_row = history_height - n.min(history_height);
        let to_skip = n.checked_sub(history_height).unwrap_or(0) as usize;

        // render history
        self.history
            .iter()
            .skip(to_skip)
            .enumerate()
            .try_for_each(|(i, msg)| {
                stdout.queue(MoveTo(x, first_row + i as u16))?;
                stdout.queue(Print(&msg[..msg.len().min(width as usize)]))?;
                Ok::<(), std::io::Error>(())
            })?;
        // render separator
        stdout.queue(MoveTo(x, y + height - 2))?;
        stdout.queue(Print(&self.horizontal_separator))?;
        // render prompt
        {
            stdout.queue(MoveTo(x, y + height - 1))?;
            let start = self.prompt.len().checked_sub(width as usize).unwrap_or(0);
            let end = (start + width as usize).min(self.prompt.len());
            stdout.queue(Print(&self.prompt[start..end]))?;
            // put cursor behind prompt
            stdout.queue(MoveTo(x + end as u16, y + height - 1))?;
        }
        Ok(())
    }
}
