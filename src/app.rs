use crossbeam::channel::{select, Receiver};
use regex::{self, Regex};
use std::io;
use termion::clear;
use termion::event::Key;
use termion::style;
use termion::terminal_size;

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Normal,
    Search,
}

pub struct App<R: io::BufRead, W: io::Write> {
    raw_buffer: Vec<String>,
    filtered_buffer: Vec<String>,
    filter: Regex,
    input: R,
    output: W,
    keys: Receiver<Key>,
    query: Vec<char>,
    mode: Mode,
}

impl<R, W> App<R, W>
where
    R: io::BufRead,
    W: io::Write,
{
    ///
    pub fn new(input: R, output: W, keys: Receiver<Key>) -> Self {
        let re = Regex::new(".*").unwrap();

        App {
            raw_buffer: Vec::new(),
            filtered_buffer: Vec::new(),
            filter: re,
            input: input,
            output: output,
            keys: keys,
            query: Vec::new(),
            mode: Mode::Normal,
        }
    }

    // Read events
    fn iterate_over_keys(&mut self) {
        let mut append_query = false;
        loop {
            select! {
                recv(self.keys) -> key => {
                     match key {
                        Ok(key) => {
                            match (self.mode, key) {
                                // No mather which mode, ctrl-c will stop the program.
                                (_, Key::Ctrl('c')) => {
                                    return
                                },

                                // Going into search mode.
                                (Mode::Normal, Key::Char('/')) => {
                                    self.mode = Mode::Search;
                                },

                                // We don't support multi-line search.
                                (Mode::Search, Key::Char('\n')) => {}

                                (Mode::Search, Key::Char(n)) => {
                                    self.query.push(n);
                                    self.redraw();
                                },

                                // Leaving search mode.
                                (Mode::Search, Key::Esc) => {
                                    self.mode = Mode::Normal;
                                    self.query = Vec::new();
                                    self.redraw();
                                },

                                (_, _) => {},
                            }
                        }
                        _ => {},
                        Err(e) => {println!("{}", e);},
                    }
                }
            }
        }
    }

    pub fn start(&mut self) {
        write!(self.output, "{}", clear::All);
        self.output.flush();

        self.read_input();
        self.iterate_over_keys();
    }

    fn read_input(&mut self) {
        loop {
            let mut line = String::new();
            let n = self
                .input
                .read_line(&mut line)
                .expect("Failed to read input");

            if n == 0 {
                return;
            }
            self.raw_buffer.push(line);
            self.redraw();
        }
    }

    fn redraw(&mut self) {
        let (width, height) = terminal_size().unwrap();
        write!(self.output, "{}", clear::All);
        self.output.flush();

        for (i, line) in self.raw_buffer.iter().rev().enumerate() {
            write!(
                self.output,
                "{}{}",
                termion::cursor::Goto(1, height - i as u16),
                line
            );
            self.output.flush();

            if i == height as usize {
                break;
            }
        }

        let spaces: usize = width as usize - self.query.len();

        let filling = vec![' '; spaces];

        let query: String = self.query.iter().collect();
        let filling: String = filling.iter().collect();

        write!(
            self.output,
            "{}{}{}{}{}",
            termion::cursor::Goto(1, height),
            style::Invert,
            query,
            filling,
            style::Reset
        );
        self.output.flush();
    }

    /// Set filter.
    pub fn filter(&mut self, query: &str) -> Result<(), regex::Error> {
        let re = format!(".*{}.*", query);
        match Regex::new(&re) {
            Ok(re) => {
                self.filter = re;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
