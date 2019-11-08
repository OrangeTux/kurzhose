use crate::error;
use crate::state;

use crossbeam::channel::{select, Receiver};
use regex::{self, Regex};
use std::fmt;
use std::io;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::thread;
use termion::clear;
use termion::color;
use termion::event::Key;
use termion::style;
use termion::terminal_size;

pub type Result<T> = std::result::Result<T, error::AppError>;

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

pub struct App<W: io::Write> {
    raw_buffer: Arc<Mutex<Vec<String>>>,
    output: W,
    state: Arc<Mutex<state::State>>,
}

impl<W> App<W>
where
    W: io::Write,
{
    ///
    pub fn new(output: W) -> Self {
        App {
            raw_buffer: Arc::new(Mutex::new(Vec::new())),
            output: output,
            state: Arc::new(Mutex::new(state::State::new())),
        }
    }

    // Return a footer that is as wide as the output is. The footer is a single line that spans
    // the width of the shell.  The query that has been searched for is left on the line, while the
    // current mode is printed at the right corner. It looks something like this.
    //
    //      <query> .........<mode>
    fn footer(&self, width: usize) -> String {
        let mut footer = String::new();
        let state = self.state.clone();
        let state = state.lock().unwrap();

        let mode = &state.mode.to_string();
        for c in state.query.clone() {
            footer.push(c);
        }

        let padding = vec![' '; width - state.query.len() - mode.chars().count() - 1];
        for c in padding {
            footer.push(c);
        }

        footer.push_str(mode);

        footer
    }

    pub fn start(&mut self, keys: Receiver<Key>) -> Result<()> {
        write!(self.output, "{}", clear::All)?;
        self.output.flush()?;

        let state = self.state.clone();
        let t = thread::spawn(move || loop {
            select! {
                    recv(keys) -> key => {
                        match key {
                            Ok(key) => {
                                if key == Key::Ctrl('c') {
                                    return
                                }
                                state.lock().unwrap().process_key(key);
                            },
                            Err(e) => {
                                return
                            }
                    }
                }
            }
        });

        let raw_buffer = self.raw_buffer.clone();
        thread::spawn(move || {
            let input = io::stdin();
            let mut input = input.lock();

            loop {
                let mut line = String::new();
                let n = input.read_line(&mut line).expect("Failed to read input");

                if n == 0 {
                    return;
                }

                raw_buffer.lock().unwrap().push(line);
            }
        });

        t.join();
        return Ok(());
    }

    fn redraw(&mut self) -> Result<()> {
        let (width, height) = terminal_size().unwrap();
        write!(self.output, "{}", clear::All)?;
        self.output.flush()?;
        let state = self.state.clone();
        let raw_buffer = self.raw_buffer.clone();

        let mut regex = String::new();
        for c in state.lock().unwrap().query.clone() {
            regex.push(c);
        }

        let regex = format!(r"(.*)(?P<m>{})(.*)", regex);
        let re = Regex::new(&regex.as_str()).unwrap();

        for (i, line) in raw_buffer.lock().unwrap().iter().rev().enumerate() {
            if re.is_match(line) {
                if i >= height as usize {
                    break;
                }

                for cap in re.captures_iter(line) {
                    write!(
                        self.output,
                        "{}{}{}{}{}{}",
                        termion::cursor::Goto(1, height - i as u16),
                        &cap[1],
                        color::Fg(color::Red),
                        &cap[2],
                        color::Fg(color::Reset),
                        &cap[3],
                    )?;
                }
                continue;
            }

            write!(
                self.output,
                "{}{}",
                termion::cursor::Goto(1, height - i as u16),
                line
            )
            .unwrap()
        }
        self.output.flush().unwrap();
        let footer = self.footer(width as usize);

        write!(
            self.output,
            "{}{}{}{}",
            termion::cursor::Goto(1, height),
            style::Invert,
            footer,
            style::Reset
        )?;
        self.output.flush()?;

        Ok(())
    }
}
