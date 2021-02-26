#![feature(map_entry_replace)]

mod error;

use std::collections::hash_map::{Entry, HashMap};
use std::fs::{self, File};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use anyhow::{anyhow, Result};
use error::{ServerError, ParseError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const BUFFER_SIZE: usize = 1024;
const ADDRESS: &str = "127.0.0.1:4000";
const SET_HEADER: &str = "GET /set?";
const GET_HEADER: &str = "GET /get?key=";
const SUCCESS_STATUS: &str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_STATUS: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const PERSIST: &str = "persist.json";

enum Request {
    Get(String),
    Set(String, String),
}

enum Response {
    GetSuccess(String),
    SetSuccess,
    NotFound,
}

#[derive(Serialize, Deserialize)]
struct Storage(HashMap<String, Value>);

pub fn server_init() -> Result<()> {
    let persisted = fs::read_to_string(PERSIST)
        .map_err(|err| ServerError::IoError(err))?;
    let mut storage = Storage(
        serde_json::from_str(&persisted)
            .unwrap_or(HashMap::new())
    );
    let listener = TcpListener::bind(ADDRESS).map_err(|_| ServerError::ConnectionError)?;

    println!("Listening on {}...", ADDRESS);

    for stream in listener.incoming() {
        let mut stream = stream?;

        match parse_request(&mut stream) {
            Ok(request) => {
                let response = handle_request(request, &mut storage);
                send_response(response, &mut stream)?;
            }
            Err(err) => {
                if let ServerError::InvalidRequest = err {
                    // got an invalid request; skip it
                    continue;
                } else {
                    return Err(anyhow!(err));
                }
            }
        }
    }

    Ok(())
}

impl Drop for Storage {
    fn drop(&mut self) {
        // Flush the contents of the HashMap to the persistence file 
        println!("Flushing data to disk...");

        let path = Path::new(PERSIST);
        let json = serde_json::to_string(&self.0).expect("Failed to serialize data"); 

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(_) => panic!("Failed to open persistence file"),
        };
    
        if let Err(_) = file.write_all(json.as_bytes()) {
            eprintln!("Failed to write to persistence file"); 
        }

        println!("Successfully flushed data to disk");
    }
}

fn handle_request(request: Request, storage: &mut Storage) -> Response {
    match request {
        Request::Get(key) => {
            if let Entry::Occupied(e) = storage.0.entry(key.clone()) {
                let val = e.get();

                println!("GET: key={}, value={}", key, val);

                Response::GetSuccess(val.to_string())
            } else {
                println!("Failed to GET value for key={}", key);
                
                Response::NotFound
            }
        },
        Request::Set(key, val) => {
            match storage.0.entry(key.clone()) {
                Entry::Occupied(o) => {
                    // overwrite the current entry
                    o.replace_entry(Value::from(val.clone()));
                }
                Entry::Vacant(v) => {
                    v.insert(Value::from(val.clone()));
                }
            }
            
            println!("SET: key={}, value={}", key, val);

            Response::SetSuccess
        }
    }
}

fn send_response(response: Response, stream: &mut TcpStream) -> Result<(), ServerError> {
    let (status_line, filename, rv) = match response {
        Response::GetSuccess(val) => (SUCCESS_STATUS, "get_success.html", Some(val)),
        Response::SetSuccess => (SUCCESS_STATUS, "set_success.html", None),
        _ => (NOT_FOUND_STATUS, "404.html", None),
    };

    let contents = fs::read_to_string(filename).map_err(|_| ServerError::NoResponseFound)?;

    let response = if rv.is_some() {
        format!("{}{}{}", status_line, contents, rv.unwrap())
    } else {
        format!("{}{}", status_line, contents)
    };

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn parse_get(request: &str) -> Result<String, ParseError> {
    let parts: Vec<&str> = request.split("key=").collect();

    if parts.len() != 2 {
        return Err(ParseError::InvalidRequest { code: 1 });
    }

    let last_part = parts.last().unwrap();

    match last_part.split_whitespace().next() {
        Some(key) => Ok(String::from(key)),
        None => Err(ParseError::MissingKey),
    }
}

fn parse_set(request: &str) -> Result<(String, String), ParseError> {
    let parts: Vec<&str> = request.split("set?").collect();

    if parts.len() != 2 {
        return Err(ParseError::InvalidRequest { code: 2 });
    }

    let last_part = parts.last().unwrap();

    match last_part.split_whitespace().next() {
        Some(kv) => {
            let kv: Vec<&str> = kv.split('=').collect();

            if kv.len() != 2 {
                return Err(ParseError::InvalidRequest { code: 3 });
            }

            Ok((
                String::from(*kv.first().unwrap()),
                String::from(*kv.last().unwrap()),
            ))
        }
        None => Err(ParseError::InvalidRequest { code: 4 }),
    }
}

fn parse_request(stream: &mut TcpStream) -> Result<Request, ServerError> {
    let mut buffer = [0; BUFFER_SIZE];
    stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer[..]);
    let request = request
        .lines()
        .take(1)
        .next()
        .ok_or(ServerError::NoRequestFound)?;

    if request.starts_with(GET_HEADER) {
        // get the key from the request
        let key = parse_get(request).map_err(|err| ServerError::ParseError {
            reason: err.to_string(),
        })?;
        Ok(Request::Get(key))
    } else if request.starts_with(SET_HEADER) {
        // get the key and value from the request
        let (key, val) = parse_set(request).map_err(|err| ServerError::ParseError {
            reason: err.to_string(),
        })?;
        Ok(Request::Set(key, val))
    } else {
        Err(ServerError::InvalidRequest)
    }
}
