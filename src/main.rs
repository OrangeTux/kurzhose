#![allow(dead_code)]
mod app;
mod error;
mod ocpp;
mod state;

extern crate nix;

pub use crate::app::App;
use crossbeam::channel;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, ErrorKind};
use std::thread;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{get_tty, is_tty};

fn main() -> app::Result<()> {
    //let args: Vec<String> = env::args().collect();
    //let input = match args.get(1) {
    //Some(file) => {
    //let f = File::open(file).expect(&format!("unable to open file '{}'", file).to_owned());
    //BufReader::new(f)
    //}
    //None => return Err(io::Error::new(ErrorKind::NotFound, "file not found"))?,
    //};
    let input = io::stdin();

    let (s1, keys) = channel::unbounded();

    let stdout = io::stdout()
        .into_raw_mode()
        .expect("failed to put STDOUT into raw mode");
    let tty = get_tty().expect("failed to obtain TTY");

    let mut app = App::new(input.lock(), stdout);

    thread::spawn(move || {
        for key in tty.keys() {
            match key {
                Ok(key) => {
                    // TODO: properly return a Result containing the error.
                    s1.send(key).unwrap()
                }
                Err(_) => {
                    // TODO: properly return a Result containing the error.
                    // I've already tried something like:
                    //    return error::AppError::IOError(e);
                    //
                    // But that fails to compile with:
                    //
                    // for key in tty.keys() {
                    //          ^^^^^^^^^^ expected enum `error::AppError`, found ()
                }
            }
        }
    });

    app.start(keys)
}

fn parse_json(data: &str) -> std::result::Result<ocpp::Message, ocpp::ParseError> {
    // Some JSON input data as a &str. Maybe this comes from the user.
    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(data)?;

    let message_type = match v.get(0) {
        Some(v) => v,
        None => return Err(ocpp::ParseError),
    };

    let message_type = match message_type.as_i64() {
        Some(v) => v,
        None => return Err(ocpp::ParseError),
    };

    let unique_id = match v.get(1) {
        Some(v) => v,
        None => return Err(ocpp::ParseError),
    };

    let unique_id = match unique_id.as_str() {
        Some(v) => v.to_string(),
        None => return Err(ocpp::ParseError),
    };

    match message_type {
        2 => {
            let action = match v.get(2) {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            let action = match action.as_str() {
                Some(v) => v.to_string(),
                None => return Err(ocpp::ParseError),
            };

            let data = match v.get(3) {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            let data = match data.as_object() {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            Ok(ocpp::Message::Call {
                unique_id: unique_id,
                action: action,
                data: data.clone(),
            })
        }
        3 => {
            let data = match v.get(2) {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            let data = match data.as_object() {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            Ok(ocpp::Message::CallResult {
                unique_id: unique_id,
                data: data.clone(),
            })
        }
        4 => {
            let error_code = match v.get(2) {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            let error_code = match error_code.as_str() {
                Some(v) => v.to_string(),
                None => return Err(ocpp::ParseError),
            };

            let error_description = match v.get(3) {
                Some(v) => v,
                None => return Err(ocpp::ParseError),
            };

            let error_description = match error_description.as_str() {
                Some(v) => v.to_string(),
                None => return Err(ocpp::ParseError),
            };

            Ok(ocpp::Message::CallError {
                unique_id: unique_id,
                error_code: error_code,
                error_description: error_description,
            })
        }
        _ => Err(ocpp::ParseError),
    }
}
