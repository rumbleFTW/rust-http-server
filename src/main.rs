use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

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
    println!("{:?}", parts);
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

fn respond_to_request(stream: &mut TcpStream, req: Request) {
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
            println!("{:?}", req);
            let _ = stream.write(buf.as_bytes());
        }
        _ => {
            let buf = b"HTTP/1.1 404 Not found\r\n\r\n";
            let _ = stream.write(buf);
        }
    }
    println!("\x1b[32mServed Request {} {}\x1b[0m", req.method, req.path);
}

fn handle_stream(mut stream: TcpStream) {
    let mut request_buffer = vec![0; 128];
    match stream.read(&mut request_buffer) {
        Ok(_) => {
            let req = parse_request(&String::from_utf8_lossy(&request_buffer));
            respond_to_request(&mut stream, req);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}

fn main() {
    let listener = TcpListener::bind(ADDRESS).unwrap();
    println!(
        "\x1b[1;34m🦀Server listening at\x1b[0m \x1b[38;5;208mhttp://{}.\x1b[0m\n\n",
        ADDRESS
    );
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // stream
                //     .set_nonblocking(true)
                //     .expect("set_nonblocking call failed");
                handle_stream(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
