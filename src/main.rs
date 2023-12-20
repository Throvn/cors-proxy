// Updated example from http://rosettacode.org/wiki/Hello_world/Web_server#Rust
// to work with Rust 1.0 beta

use reqwest;
use reqwest::blocking::Response;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_read(mut stream: &TcpStream) -> Result<String, Error> {
    let mut buf = [0u8; 4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            return Ok(String::from_utf8_lossy(&buf).into());
        }
        Err(e) => return Err(e),
    }
}

fn handle_write(mut req_response: Response, mut stream: TcpStream) {
    unsafe {
        println!(
            "{}",
            std::str::from_utf8_unchecked(
                req_response
                    .headers()
                    .get("content-type")
                    .unwrap()
                    .as_bytes()
            )
        );
    }

    let mut req_res = vec![];
    let size = req_response.copy_to(&mut req_res).unwrap();
    println!("Response size: {}", size);

    let res_header = req_response.headers().get("content-type").unwrap();

    let response: Vec<u8> = [
        b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: ",
        res_header.as_bytes(),
        b"\r\nContent-Length: ",
        req_res.len().to_string().as_bytes(),
        // req_response
        //     .headers()
        //     .get("content-length")
        //     .unwrap()
        //     .as_bytes(),
        b"\r\n\r\n",
        req_res.as_slice(),
        b"\r\n",
    ]
    .concat();
    match stream.write(response.as_slice()) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

fn handle_client(stream: TcpStream) {
    let contents = handle_read(&stream);
    if contents.is_err() {
        println!("Failed reading stream: {}", contents.err().unwrap());
        return;
    }
    let contents = contents.unwrap();
    let mut lines = contents.lines();
    let path = lines.next().unwrap().split_whitespace().nth(1).unwrap();
    let mut chars = path.chars();
    chars.next(); // skip leading '/'
    let path = chars.as_str();
    println!("Request path: {}", path);

    println!(
        "Received request: {}",
        lines.collect::<Vec<&str>>().join("\n")
    );

    let response = reqwest::blocking::get(path);
    if let Err(e) = response {
        println!("Failed to get response: {}", e);
        return;
    }
    let response = response.unwrap();

    handle_write(response, stream);
}

fn main() {
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let listener = TcpListener::bind("0.0.0.0:".to_string() + port.as_str()).unwrap();
    println!("Listening for connections on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
