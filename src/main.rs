use std::env;
use std::fs;
use std::path;
use std::thread;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use nom::AsBytes;

const ADDRESS: &str = "127.0.0.1:4221";

#[allow(dead_code)]
#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    version: String,
    user_agent: String,
}

fn parse_request(request_string: &str) -> Request {
    let parts: Vec<&str> = request_string.split_whitespace().collect();
    let http_verb = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();
    let idx = parts.iter().position(|&r| r == "User-Agent:");
    if idx.is_none() {
        return Request {
            method: http_verb,
            path: path,
            version: version,
            user_agent: "None".to_string(),
        };
    }

    return Request {
        method: http_verb,
        path: path,
        version: version,
        user_agent: parts[idx.unwrap() + 1].to_string(),
    };
}

fn respond_to_request(stream: &mut TcpStream, req: Request, base_dir: path::PathBuf) {
    if req.method == "GET" {
        match req.path.as_str() {
            "/" => {
                let buf = b"HTTP/1.1 200 OK\r\n\r\n";
                let _ = stream.write(buf);
            }
            _ if req.path.as_str().starts_with("/echo") => {
                let mut sub_path = req.path.replace("/echo", "");
                if sub_path.starts_with("/") {
                    sub_path = format!("{}", &sub_path[1..]);
                }
                let buf = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    sub_path.len(),
                    sub_path
                );
                let _ = stream.write(buf.as_bytes());
            }
            _ if req.path.as_str().starts_with("/user-agent") => {
                let buf = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    req.user_agent.len(),
                    req.user_agent
                );
                let _ = stream.write(buf.as_bytes());
            }
            _ if req.path.as_str().starts_with("/files") => {
                let mut sub_path = req.path.replace("/files", "");
                if sub_path.starts_with("/") {
                    sub_path = format!("{}", &sub_path[1..]);
                }
                let target = base_dir.join(path::PathBuf::from(&sub_path));
                let file_buf = fs::read(target);
                match file_buf {
                    Ok(file_buf) => {
                        let buf = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n", file_buf.len()
                        );
                        let _ = stream.write(&[buf.as_bytes(), file_buf.as_bytes()].concat());
                    }
                    Err(_) => {
                        let _ = stream.write(b"HTTP/1.1 404 Not found\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\n\r\n");
                    }
                }
            }
            _ => {
                let _ = stream.write(b"HTTP/1.1 404 Not found\r\n\r\n");
            }
        }
    } else if req.method == "POST" {
        match req.path.as_str() {
            _ if req.path.as_str().starts_with("/files") => {
                let mut sub_path = req.path.replace("/files", "");
                if sub_path.starts_with("/") {
                    sub_path = format!("{}", &sub_path[1..]);
                }
                let target = base_dir.join(path::PathBuf::from(&sub_path));
            }
            _ => {
                let _ = stream.write(b"HTTP/1.1 404 Not found\r\n\r\n");
            }
        }
    }
    println!("\x1b[32mServed Request {} {}\x1b[0m", req.method, req.path);
}

fn handle_stream(mut stream: TcpStream, base_dir: path::PathBuf) {
    let mut request_buffer = vec![0; 128];
    match stream.read(&mut request_buffer) {
        Ok(_) => {
            let req = parse_request(&String::from_utf8_lossy(&request_buffer));
            dbg!(&req);
            respond_to_request(&mut stream, req, base_dir);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut base_dir = path::PathBuf::from(".");
    if args.len() > 2 && args[1] == "--directory" {
        base_dir = path::PathBuf::from(args[2].clone());
    }

    let listener = TcpListener::bind(ADDRESS).unwrap();
    println!(
        "\x1b[1;34m🦀Server listening at\x1b[0m \x1b[38;5;208mhttp://{}.\x1b[0m\n\n",
        ADDRESS
    );
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let base_dir = base_dir.clone();
                thread::spawn(move || handle_stream(stream, base_dir));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
