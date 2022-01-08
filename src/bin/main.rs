use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use multi_thread_web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let pool = ThreadPool::new(num_cpus::get()).unwrap();

    listener
        .incoming()
        .for_each(|stream| pool.execute(|| handle_connection(stream.unwrap())).unwrap());
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let status_line = if buffer.starts_with(get) {
        "HTTP/1.1 200 OK"
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 404 NOT FOUND"
    };

    let contents = "{\"hello\": \"world\"}";

    let response = format!(
        "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
