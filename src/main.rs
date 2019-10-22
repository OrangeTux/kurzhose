mod app;
mod ocpp;

extern crate nix;

pub use crate::app::App;
use crossbeam::channel;
use serde_json::Value;
use std::io;
use std::thread;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{get_tty, is_tty};

fn main() -> () {
    let stdin = io::stdin();
    if is_tty(&stdin) {
        panic!("Missing input")
    }

    let (s1, r1) = channel::unbounded();

    let stdout = io::stdout().into_raw_mode().unwrap();
    let tty = get_tty().unwrap();

    let mut app = App::new(stdin.lock(), stdout, r1);

    thread::spawn(move || {
        for key in tty.keys() {
            match key {
                Ok(key) => {
                    s1.send(key);
                }
                Err(e) => println!("{:?}", e),
            }
        }
    });

    app.start();
}

fn parse_json(data: &str) -> Result<ocpp::Message, ocpp::ParseError> {
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
