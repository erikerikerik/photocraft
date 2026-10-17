//! An in-process stand-in for Craft AI Manager: a `TcpListener` on `127.0.0.1:0`, a thread, and
//! scripted raw replies. It records every request so tests can assert on headers and bodies.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

/// A request the fake manager accepted.
#[derive(Clone, Debug)]
pub struct Captured {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Captured {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value.as_str())
    }
}

/// A scripted reply. `early` answers as soon as the request head has arrived and then reads the
/// rest of the body without keeping it (what Craft AI Manager does with a stale token or an
/// oversized upload, BF-01).
struct Scripted {
    bytes: Vec<u8>,
    early: bool,
}

type Replies = Arc<Mutex<VecDeque<Scripted>>>;

pub struct FakeServer {
    addr: SocketAddr,
    replies: Replies,
    captured: Arc<Mutex<Vec<Captured>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl FakeServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("the fake manager must be able to bind a loopback port");
        let addr = listener.local_addr().expect("a bound loopback listener has an address");
        listener.set_nonblocking(true).expect("a loopback listener can be non-blocking");
        let replies: Replies = Arc::new(Mutex::new(VecDeque::new()));
        let captured = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let replies = Arc::clone(&replies);
            let captured = Arc::clone(&captured);
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                let mut children: Vec<JoinHandle<()>> = Vec::new();
                loop {
                    if stop.load(Ordering::Relaxed) {
                        break;
                    }
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let replies = Arc::clone(&replies);
                            let captured = Arc::clone(&captured);
                            children.push(std::thread::spawn(move || serve_one(stream, replies, captured)));
                        }
                        Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => break,
                    }
                }
                for child in children {
                    let _ = child.join();
                }
            })
        };
        Self { addr, replies, captured, stop, thread: Some(thread) }
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Queue one raw reply, written in answer to the next request.
    pub fn queue(&self, reply: Vec<u8>) {
        self.replies.lock().unwrap_or_else(PoisonError::into_inner).push_back(Scripted { bytes: reply, early: false });
    }

    /// Queue a reply sent right after the next request's head, before its body is read: the write
    /// side is then closed and the rest of the body is read and thrown away, which is what Craft
    /// AI Manager does (BF-01). The upload finishes, but the answer the client gets is this one.
    pub fn queue_early(&self, reply: Vec<u8>) {
        self.replies.lock().unwrap_or_else(PoisonError::into_inner).push_back(Scripted { bytes: reply, early: true });
    }

    pub fn queue_json(&self, status: u16, body: &str) {
        self.queue(response(status, "application/json", body.as_bytes()));
    }

    /// The requests seen so far, in order.
    pub fn captured(&self) -> Vec<Captured> {
        self.captured.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub fn capture_count(&self) -> usize {
        self.captured.lock().unwrap_or_else(PoisonError::into_inner).len()
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve_one(mut stream: TcpStream, replies: Replies, captured: Arc<Mutex<Vec<Captured>>>) {
    // On Windows an accepted socket inherits the listener's non-blocking mode, so a read that has
    // nothing yet fails instead of waiting; the real manager puts the socket back to blocking for
    // the same reason.
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let Some((mut request, content_length)) = read_head(&mut stream) else { return };
    let reply = replies
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .pop_front()
        .unwrap_or_else(|| Scripted { bytes: response(500, "application/json", br#"{"error": "the fake manager had no scripted reply"}"#), early: false });
    if reply.early {
        // Answer from the head, half-close (the answer is finished), then read the rest of the body
        // the client announced and throw it away - what Craft AI Manager does (BF-01). Closing with
        // the body unread resets the connection, and on Windows the reset throws away the answer
        // the client has not read yet: it would report a lost connection instead of the 401.
        let _ = stream.write_all(&reply.bytes);
        let _ = stream.flush();
        let _ = stream.shutdown(Shutdown::Write);
        let announced = content_length.saturating_sub(request.body.len());
        // Record the head before waiting for the rest of the body. A client that reads this early
        // answer can finish while those bytes are still arriving, and a test that asks what the
        // fake manager saw would find nothing yet.
        captured.lock().unwrap_or_else(PoisonError::into_inner).push(request);
        discard(&mut stream, announced);
        return;
    }
    read_body(&mut stream, &mut request.body, content_length);
    captured.lock().unwrap_or_else(PoisonError::into_inner).push(request);
    let _ = stream.write_all(&reply.bytes);
    let _ = stream.flush();
    let _ = stream.shutdown(Shutdown::Both);
}

/// The request line and headers (plus whatever body bytes came with them) and the announced
/// `Content-Length`, or `None` if the head never arrives whole.
fn read_head(stream: &mut TcpStream) -> Option<(Captured, usize)> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 8 * 1024];
    let header_end = loop {
        if buf.len() > 64 * 1024 {
            return None;
        }
        match stream.read(&mut chunk) {
            Ok(0) => return None,
            Ok(read) => {
                buf.extend_from_slice(&chunk[..read]);
                if let Some(at) = find_double_crlf(&buf) {
                    break at;
                }
            }
            Err(_) => return None,
        }
    };
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let mut words = lines.next().unwrap_or_default().split(' ');
    let method = words.next().unwrap_or_default().to_string();
    let path = words.next().unwrap_or_default().to_string();
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let (name, value) = (name.trim().to_string(), value.trim().to_string());
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().unwrap_or(usize::MAX);
            }
            headers.push((name, value));
        }
    }
    let body = buf[header_end + 4..].to_vec();
    Some((Captured { method, path, headers, body }, content_length))
}

/// Read and throw away the rest of a request the fake manager has already answered, so closing the
/// socket does not reset the connection. Bounded: the length is the caller's to announce.
fn discard(stream: &mut TcpStream, count: usize) {
    let mut chunk = [0u8; 8 * 1024];
    let mut left = count;
    while left > 0 {
        let want = left.min(chunk.len());
        match stream.read(&mut chunk[..want]) {
            Ok(0) => break,
            Ok(read) => left = left.saturating_sub(read),
            Err(_) => break,
        }
    }
}

/// The rest of a `Content-Length` body.
fn read_body(stream: &mut TcpStream, body: &mut Vec<u8>, content_length: usize) {
    let mut chunk = [0u8; 8 * 1024];
    while body.len() < content_length {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => body.extend_from_slice(&chunk[..read]),
            Err(_) => break,
        }
    }
}

fn find_double_crlf(haystack: &[u8]) -> Option<usize> {
    haystack.windows(4).position(|window| window == b"\r\n\r\n")
}

/// A complete raw HTTP reply.
pub fn response(status: u16, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut out =
        format!("HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", reason(status), body.len())
            .into_bytes();
    out.extend_from_slice(body);
    out
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Content Too Large",
        500 => "Internal Server Error",
        _ => "Answer",
    }
}
