use std::io::{stdin, stdout, Write, BufRead, BufReader, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::put;

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

    if stream.set_read_timeout(Some(Duration::from_millis(500))).is_err() {
        eprintln!("Failed to set socket timeout");
        return 1;
    }

    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Receiver thread
    let mut rx_stream = stream.try_clone().expect("TcpStream::try_clone failed");
    let recv = thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            match rx_stream.read(&mut buf) {
                Ok(0) => {
                    put(|o| writeln!(o, "\nConnection closed by remote"));
                    r.store(false, Ordering::SeqCst);
                    break;
                }
                Ok(n) => {
                    put(|o| {
                        write!(o, "\n[recv {} bytes] ", n)?;
                        o.write_all(&buf[..n])?;
                        write!(o, "\n> ")
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                           || e.kind() == std::io::ErrorKind::TimedOut => {
                    if !r.load(Ordering::SeqCst) { break; }
                    continue;
                }
                Err(_) => {
                    put(|o| writeln!(o, "\nConnection lost"));
                    r.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }
    });

    println!("Connected to {} ({})", args[1], target_addr);

    let stdin = BufReader::new(stdin());
    for line in stdin.lines() {
        put(|o| write!(o, "> "));
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if !running.load(Ordering::SeqCst) { break; }
        if line == "/quit" { break; }

        let mut data = line.into_bytes();
        data.push(b'\n');
        if let Err(_) = stream.write_all(&data) {
            let _ = stdout().flush(); // ensure "> " is visible
            eprintln!("Send failed");
            running.store(false, Ordering::SeqCst);
            break;
        }
    }

    running.store(false, Ordering::SeqCst);
    drop(stream);
    recv.join().ok();

    0
}
