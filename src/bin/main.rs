use std::time::Duration;
use std::{fs, thread};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use basic_rust_server::ThreadPool;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);


    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_stream(stream)
        });
    }
}

fn handle_stream(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let ( status_line, filename ) =
        if buffer.starts_with(get) {
            ("200 OK", "static/index.html")
        } else if buffer.starts_with(sleep) {
            thread::sleep(Duration::from_secs(15));
            ("200 OK", "static/index.html")
        } else {
            ("404 NOT FOUND", "static/error.html")
        };

    let content = fs::read_to_string(filename).unwrap();
    let response = format!(
        "HTTP/1.1 {} \r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content.len(),
        content
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
