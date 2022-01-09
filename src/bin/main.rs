use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use multi_thread_web_server::ThreadPool;

struct MyRoute<'a> {
    name: &'a str,
    path: &'a str,
    status: &'a str,
    contents: &'a str,
    callback: Option<Box<dyn FnOnce() + Send + 'static>>,
}

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

    let routes = vec![
        MyRoute {
            name: "get",
            path: "GET / HTTP/1.1\r\n",
            status: "HTTP/1.1 200 OK",
            contents: "{\"hello\": \"world\"}",
            callback: None,
        },
        MyRoute {
            name: "sleep",
            path: "GET /sleep HTTP/1.1\r\n",
            status: "HTTP/1.1 200 OK",
            contents: "{\"hello\": \"world\"}",
            callback: Some(Box::new(|| thread::sleep(Duration::from_secs(5)))),
        },
    ];
    let not_found = MyRoute {
        name: "404",
        path: "",
        status: "HTTP/1.1 404 NOT FOUND",
        contents: "404",
        callback: None,
    };

    let route = routes
        .into_iter()
        .find(|item| buffer.starts_with(item.path.as_bytes()));

    let result = route.unwrap_or_else(|| not_found);
    if let Some(cb) = result.callback {
        cb();
    }
    println!("Match route: {} and path: {}", result.name, result.path);

    let response = format!(
        "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        result.status,
        result.contents.len(),
        result.contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
