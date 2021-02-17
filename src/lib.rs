use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream}; 

use anyhow::{Context, Result};
use thiserror::Error;

const BUFFER_SIZE: usize = 1024;
const ADDRESS: &str = "127.0.0.1:4000";
const SET_HEADER: &str = "GET /set?";
const GET_HEADER: &str = "GET /get?key="; 
const SUCCESS_STATUS: &str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_STATUS: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";

enum Request {
    GET(String),
    SET(String, String),
}

#[derive(Error, Debug)]
enum ServerError {
    #[error("There was an error parsing your request: {reason:?}")]
    ParseError { reason: String },
}

#[derive(Error, Debug)]
enum ParseError {
    #[error("Request was improperly formatted.")]
    InvalidRequest, 
    #[error("No key found in request.")]
    MissingKey,
}

fn server_init() -> Result<()> {
    let listener = TcpListener::bind(ADDRESS).context("Failed to bind to address")?;

    for stream in listener.incoming() {
        let stream = stream.context("Failed to connect to client.")?;
        let request = parse_request(stream)?;
        handle_request(request)?; 
    }

    Ok(())
}

fn parse_get(request: &str) -> Result<String, ParseError> {
    let parts: Vec<&str> = request.split("key=").collect();
    
    if parts.len() != 2 { return Err(ParseError::InvalidRequest); }

    let last_part = parts.last().unwrap();
    
    match last_part.split_whitespace().next() {
        Some(key) => Ok(String::from(key)),
        None => Err(ParseError::MissingKey)
    }
}

fn parse_set(request: &str) -> Result<(String, String), ParseError> {
    let parts: Vec<&str> = request.split("set?").collect();

    if parts.len() != 2 { return Err(ParseError::InvalidRequest); }

    let last_part = parts.last().unwrap();

    match last_part.split_whitespace().next() {
        Some(kv) => {
            let kv: Vec<&str> = kv.split('=').collect(); 

            if kv.len() != 2 { return Err(ParseError::InvalidRequest); }

            Ok((String::from(*kv.first().unwrap()), String::from(*kv.last().unwrap())))
        },
        None => Err(ParseError::InvalidRequest)
    }
}

fn parse_request(mut stream: TcpStream) -> Result<Request, ParseError> {
    let mut buffer = [0; BUFFER_SIZE];
    stream.read(&mut buffer).expect("Failed to read client request.");
    
    let request = String::from_utf8_lossy(&buffer[..]);
    let request = match request.lines().take(1).next() {
        Some(req) => req,
        None => panic!("Received no request from client."),
    };

    // println!("Request: {}", request);

    if request.starts_with(GET_HEADER) {
        // get the key from the request 
        let key = parse_get(request)?;
        Ok(Request::GET(key))
    } else if request.starts_with(SET_HEADER) {
        // get the key and value from the request 
        let (key, val) = parse_set(request)?; 
        Ok(Request::SET(key, val))
    } else {
        Err(ParseError::InvalidRequest)
    }
}
