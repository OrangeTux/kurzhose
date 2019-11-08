use crate::app;
use crate::error;
use std::fmt;
use termion::event::Key;

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Normal,
    Search,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct State {
    pub mode: Mode,
    pub query: Vec<char>,
}

impl State {
    pub fn new() -> Self {
        State {
            mode: Mode::Normal,
            query: Vec::new(),
        }
    }

    pub fn process_key(&mut self, key: Key) -> app::Result<()> {
        match (self.mode, key) {
            // No mather which mode, ctrl-c will stop the program.
            (_, Key::Ctrl('c')) => return Ok(()),

            // Going into search mode.
            (Mode::Normal, Key::Char('/')) => {
                self.mode = Mode::Search;
                Ok(())
            }

            // We don't support multi-line search.
            (Mode::Search, Key::Char('\n')) => Ok(()),
            (Mode::Search, Key::Backspace) => {
                self.query.pop();
                Ok(())
            }

            (Mode::Search, Key::Char(n)) => {
                self.query.push(n);
                Ok(())
            }

            // Leaving search mode.
            (Mode::Search, Key::Esc) => {
                self.mode = Mode::Normal;
                self.query = Vec::new();
                Ok(())
            }

            (_, _) => Ok(()),
        }
    }
}
