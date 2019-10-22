use crossbeam::channel::{select, Receiver};
use regex::{self, Regex};
use std::io;
use termion::clear;
use termion::event::Key;
use termion::style;
use termion::terminal_size;

pub struct App<R: io::BufRead, W: io::Write> {
    raw_buffer: Vec<String>,
    filtered_buffer: Vec<String>,
    filter: Regex,
    input: R,
    output: W,
    keys: Receiver<Key>,
    query: Vec<char>,
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
        }
    }

    // Read events
    fn iterate_over_keys(&mut self) {
        let mut append_query = false;
        loop {
            select! {
                recv(self.keys) -> key => {
                     match key {
                        Ok(Key::Char('/')) => append_query = true,
                        Ok(Key::Esc) => {
                            append_query = false;
                            self.query = Vec::new();
                            self.redraw();
                        },
                        Ok(Key::Ctrl('c')) => return,
                        Ok(Key::Char(n)) => {
                            if append_query {
                                self.query.push(n);
                                self.redraw();
                            }
                        },
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
        self.iterate_over_keys();
    }

    fn redraw(&mut self) {
        let (width, height) = terminal_size().unwrap();
        write!(self.output, "{}", clear::All);
        self.output.flush();
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
