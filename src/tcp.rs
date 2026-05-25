use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::output::{prompt, recv, send, status};

enum Event { Input(String), Close }

fn connect_timeout(addr: std::net::SocketAddr, timeout: Duration) -> std::io::Result<TcpStream> {
    let (tx, rx) = mpsc::channel();
    let a = addr;
    thread::spawn(move || { let _ = tx.send(TcpStream::connect(a)); });
    rx.recv_timeout(timeout)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "connect timed out"))
        .and_then(|r| r)
}

pub fn run(args: &[String]) -> i32 {
    if args.len() < 3 {
        eprintln!("Usage: net-helper -t <ip|domain> <port>");
        return 1;
    }

    let target_str = format!("{}:{}", args[1], args[2]);
    let target_addr = match target_str.to_socket_addrs() {
        Ok(mut iter) => match iter.next() {
            Some(a) => a,
            None => { eprintln!("Failed to resolve: {}", args[1]); return 1; }
        },
        Err(_) => { eprintln!("Failed to resolve: {}", args[1]); return 1; }
    };

    let mut stream = match connect_timeout(target_addr, Duration::from_secs(5)) {
        Ok(s) => s,
        Err(_) => { eprintln!("Failed to connect to {}:{}", args[1], args[2]); return 1; }
    };
    stream.set_read_timeout(Some(Duration::from_millis(500))).ok();

    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let (tx, rx) = mpsc::channel::<Event>();

    // stdin adapter
    let tx_in = tx.clone();
    let stdin_rx = crate::input::reader();
    thread::spawn(move || { for line in stdin_rx { if tx_in.send(Event::Input(line)).is_err() { break; } } });

    // receiver
    let mut rx_stream = stream.try_clone().expect("TcpStream::try_clone failed");
    let tx_close = tx;
    thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            match rx_stream.read(&mut buf) {
                Ok(0)  => { status("Connection closed by remote"); r.store(false, Ordering::SeqCst); let _ = tx_close.send(Event::Close); break; }
                Ok(n)  => recv(&buf[..n]),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                          || e.kind() == std::io::ErrorKind::TimedOut => { if !r.load(Ordering::SeqCst) { break; } continue; }
                Err(_) => { status("Connection lost"); r.store(false, Ordering::SeqCst); let _ = tx_close.send(Event::Close); break; }
            }
        }
    });

    println!("Connected to {} ({})", args[1], target_addr);

    prompt();
    loop {
        match rx.recv() {
            Ok(Event::Input(line)) => {
                if !running.load(Ordering::SeqCst) { break; }
                if line == "/quit" { break; }
                let mut data = line.into_bytes();
                data.push(b'\n');
                if stream.write_all(&data).is_err() {
                    eprintln!("Send failed");
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                send(&data);
            }
            Ok(Event::Close) => break,
            Err(_) => break,
        }
    }

    running.store(false, Ordering::SeqCst);
    drop(stream);
    0
}
