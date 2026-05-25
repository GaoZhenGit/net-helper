use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::{put, size_fmt, write_prefixed};

enum Event {
    Input(String),
    Close,
}

fn connect_timeout(addr: std::net::SocketAddr, timeout: Duration) -> std::io::Result<TcpStream> {
    let (tx, rx) = mpsc::channel();
    let a = addr;
    thread::spawn(move || {
        let _ = tx.send(TcpStream::connect(a));
    });
    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "connect timed out",
        )),
    }
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
            None => {
                eprintln!("Failed to resolve: {}", args[1]);
                return 1;
            }
        },
        Err(_) => {
            eprintln!("Failed to resolve: {}", args[1]);
            return 1;
        }
    };

    let mut stream = match connect_timeout(target_addr, Duration::from_secs(5)) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Failed to connect to {}:{}", args[1], args[2]);
            return 1;
        }
    };

    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok();

    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let (tx, rx) = mpsc::channel::<Event>();

    // Stdin → Event adapter
    let tx_input = tx.clone();
    let stdin_rx = crate::input::reader();
    thread::spawn(move || {
        for line in stdin_rx {
            if tx_input.send(Event::Input(line)).is_err() {
                break;
            }
        }
    });

    let mut rx_stream = stream.try_clone().expect("TcpStream::try_clone failed");
    let tx_close = tx;
    thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            match rx_stream.read(&mut buf) {
                Ok(0) => {
                    put(|o| writeln!(o, "\r\x1b[2KConnection closed by remote"));
                    r.store(false, Ordering::SeqCst);
                    let _ = tx_close.send(Event::Close);
                    break;
                }
                Ok(n) => {
                    put(|o| {
                        write!(o, "\r\x1b[2K[recv {}]", size_fmt(n))?;
                        o.write_all(b"\n")?;
                        write_prefixed(o, &buf[..n], "<- ")?;
                        write!(o, "> ")
                    });
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if !r.load(Ordering::SeqCst) {
                        break;
                    }
                    continue;
                }
                Err(_) => {
                    put(|o| writeln!(o, "\r\x1b[2KConnection lost"));
                    r.store(false, Ordering::SeqCst);
                    let _ = tx_close.send(Event::Close);
                    break;
                }
            }
        }
    });

    println!("Connected to {} ({})", args[1], target_addr);

    put(|o| write!(o, "> "));
    loop {
        match rx.recv() {
            Ok(Event::Input(line)) => {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                if line == "/quit" {
                    break;
                }

                let mut data = line.into_bytes();
                data.push(b'\n');

                if let Err(_) = stream.write_all(&data) {
                    eprintln!("Send failed");
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                let len = data.len();
                put(|o| {
                    write!(o, "\r\x1b[2K[send {}]", size_fmt(len))?;
                    o.write_all(b"\n")?;
                    write_prefixed(o, &data, "-> ")?;
                    write!(o, "> ")
                });
            }
            Ok(Event::Close) => break,
            Err(_) => break,
        }
    }

    running.store(false, Ordering::SeqCst);
    drop(stream);
    0
}
