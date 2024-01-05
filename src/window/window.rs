use anyhow::Result;
use crossterm::event::KeyEvent;

pub enum WindowAction {
    None,
    Terminate,
}

pub trait Window {
    fn handle_keypress(&mut self, event: KeyEvent) -> Result<WindowAction>;
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<WindowAction>;
    fn render(&self) -> Result<()>;
}
