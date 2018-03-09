use std::thread::spawn;
use std::net::{ TcpListener, TcpStream };
use std::io::prelude::*;

use std::io::BufReader;

struct Request<'a> {
    dir: &'a str,
    host: &'a str,
}

fn client_handler(client: TcpStream) {
    let mut reader = BufReader::new(client);

    println!("User: '{}'", get_address(reader.get_mut()));
    if let Some(data) = read_request(&mut reader) {
        println!("Data:\n{}", data);
        if let Some(request) = parse_request(&data) {
            handle_request(request, reader.get_mut());
        } else {
            send_response(400, "<title>Ajaj</title><h1>Ajaj nu blev det knas</h1>", reader.get_mut())
        }
    }
    println!("Closed\n");
}

fn get_address(client: &TcpStream) -> String{
    if let Ok(adr) = client.peer_addr() {
        adr.to_string()
    } else {
        "Unknown ip".to_owned()
    }
}

fn handle_request(request: Request, client: &mut TcpStream) {
    let address = get_address(client);

    let (code, msg) = match request.dir {
        "/" | "/index.html" => (200, format!("Hej hopp '{}'", address)),
        dir => (404, format!("Nä, {} finns inte!", dir)),
    };
    send_response(code, &format!("<title>{}</title><h1>{}</h1>", code, msg), client);
}

fn send_response(status_code: u32, content: &str, client: &mut TcpStream) {
    let status = match status_code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        418 => "I'm a teapot",
        _ => unimplemented!("Status code not implemented")
    };

    let msg = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Encoding: UTF-8\r\nServer: HemmaBygge\r\n\r\n{}",
        status_code, status, content
    );
    let mut bytes = msg.into_bytes();

    if let Err(err) = client.write_all(bytes.as_mut()) {
        println!("Failed to send response with error: {}", err.to_string());
    }
}

fn parse_request(data: &str) -> Option<Request> {
    let data = data.trim_right();
    let mut dir = None;
    let mut host = None;

    for line in data.lines() {
        let mut i = line.split(" ");
        let key = if let Some(k) = i.next() { k } else { continue; };

        let value = if let Some(k) = i.next() { k } else { continue; };

        if key == "GET" {
            dir = Some(value);
        } else if key == "Host:" {
            host = Some(value);
        }
    }


    if let (Some(dir), Some(host)) = (dir, host) {
        Some(Request{
            dir,
            host
        })
    } else {
        None
    }
}

fn read_request(reader: &mut BufReader<TcpStream>) -> Option<String> {
    let mut lines = String::new();
    while let Ok(bytes_read) = reader.read_line(&mut lines) {
        if bytes_read == 0 {
            return None
        }
        if lines.len() > 4 && lines[lines.len() - 4..] == *"\r\n\r\n" {
            return Some(lines)
        }
    }
    None
}

fn main() {
    let server = TcpListener::bind("0.0.0.0:8080").expect("Failed to open server!!!");
    for c in server.incoming() {
        if let Ok(client) = c {
            spawn(||
                client_handler(client)
            );
        }
    }

}
