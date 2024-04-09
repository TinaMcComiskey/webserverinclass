use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::io::{BufRead, Write, BufReader};

fn discard_request(tcp_stream: &mut TcpStream) {
    let mut reader = BufReader::new(tcp_stream);
    let mut line = String::new();
    loop {
        reader.read_line(&mut line).unwrap();
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return;
        }
        eprintln!("{}", trimmed);
        line.clear();
    }
}

fn main() {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let socket_addr = SocketAddrV4::new(localhost, 3000);
    eprintln!("server starts: {}", socket_addr);
    let tcp_listener = TcpListener::bind(socket_addr).unwrap();
    loop {
        let (mut tcp_stream, addr) = tcp_listener.accept().unwrap();
        eprintln!("connection from {}", addr);
        discard_request(&mut tcp_stream);
        write!(tcp_stream, "HTTP/1.0 200 OK\r\n").unwrap();
        write!(tcp_stream, "Content-Type: text/html; charset=utf-8\r\n").unwrap();
        write!(tcp_stream, "\r\n").unwrap();
        write!(tcp_stream, "<html>").unwrap();
        write!(tcp_stream, "<head>").unwrap();
        write!(tcp_stream, "<meta charset=\"UTF-8\"/>").unwrap();
        write!(tcp_stream, "<title>sussy bakka ඞ</title>").unwrap();
        write!(tcp_stream, "</head>").unwrap();
        write!(tcp_stream, "<body>").unwrap();
        write!(tcp_stream, "<em>sussy bakka ඞ</em>").unwrap();
        write!(tcp_stream, "</body>").unwrap();
        write!(tcp_stream, "</html>").unwrap();
        tcp_stream.flush().unwrap();
    }
}