use std::io::{Read, Result, Write};
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use crate::tls::TlsStream;

fn connect_timeout(addr: std::net::SocketAddr, timeout: Duration) -> std::io::Result<TcpStream> {
    let (tx, rx) = mpsc::channel();
    let a = addr;
    thread::spawn(move || { let _ = tx.send(TcpStream::connect(a)); });
    rx.recv_timeout(timeout)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "connect timed out"))
        .and_then(|r| r)
}

pub fn run(args: &[String]) -> i32 {
    let sub = &args[1..];
    let tls = sub.iter().any(|a| a == "-tls" || a == "--tls");
    let positional: Vec<&str> = sub.iter()
        .map(|s| s.as_str())
        .filter(|s| !s.starts_with('-'))
        .collect();
    if positional.len() < 2 {
        eprintln!("Usage: net-helper -t [-tls] <ip|domain> <port>");
        return 1;
    }
    let (host, port) = (positional[0], positional[1]);

    let ipv6 = sub.iter().any(|a| a == "-ipv6" || a == "-6");
    let addrs = crate::dns::resolve(host, port.parse().unwrap_or(0), ipv6);
    if addrs.is_empty() { eprintln!("Failed to resolve: {}", host); return 1; }
    let mut raw = None;
    for addr in &addrs {
        if let Ok(s) = connect_timeout(*addr, Duration::from_secs(2)) { raw = Some(s); break; }
    }
    let raw = match raw { Some(s) => s, None => { eprintln!("Failed to connect to {}:{}", host, port); return 1; } };

    let running = Arc::new(AtomicBool::new(true));

    if tls {
        let tls = match crate::tls::TlsStream::connect(raw, host) {
            Ok(s) => s, Err(e) => { eprintln!("TLS handshake failed: {}", e); return 1; }
        };
        crate::console::println(&format!("Connected to {} ({}:{}) [TLS]", host, host, port));
        // Arc<Mutex<>> for shared read/write
        let stream = Arc::new(Mutex::new(tls));
        let rx = Arc::clone(&stream);
        let close_rx = crate::net::spawn_receiver(TlsReader(rx), &running);
        crate::net::interactive(TlsWriter(stream), running, close_rx)
    } else {
        raw.set_read_timeout(Some(Duration::from_millis(500))).ok();
        crate::console::println(&format!("Connected to {}:{})", host, port));
        let rx = raw.try_clone().expect("TcpStream::try_clone failed");
        let close_rx = crate::net::spawn_receiver(rx, &running);
        crate::net::interactive(raw, running, close_rx)
    }
}

// ── TLS delegation wrappers ──

struct TlsReader(Arc<Mutex<TlsStream>>);
struct TlsWriter(Arc<Mutex<TlsStream>>);

impl Read for TlsReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> { self.0.lock().unwrap().read(buf) }
}
impl Write for TlsWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> { self.0.lock().unwrap().write(buf) }
    fn flush(&mut self) -> Result<()> { self.0.lock().unwrap().flush() }
}
