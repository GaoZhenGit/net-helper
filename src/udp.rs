use std::io::{stdin, BufRead, BufReader};
use std::net::{UdpSocket, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::put;

pub fn run(args: &[String]) -> i32 {
    if args.len() < 3 {
        eprintln!("Usage: net-helper -u <ip|domain> <port>");
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

    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Failed to create UDP socket");
            return 1;
        }
    };

    if sock.set_read_timeout(Some(Duration::from_millis(500))).is_err() {
        eprintln!("Failed to set socket timeout");
        return 1;
    }

    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Receiver thread
    let rx_sock = sock.try_clone().expect("UdpSocket::try_clone failed");
    let recv = thread::spawn(move || {
        let mut buf = [0u8; 65536];
        while r.load(Ordering::SeqCst) {
            match rx_sock.recv_from(&mut buf) {
                Ok((n, from)) => {
                    put(|o| {
                        write!(o, "\n[recv {} {}B] ", from, n)?;
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
                    if !r.load(Ordering::SeqCst) { break; }
                    continue;
                }
            }
        }
    });

    println!("UDP connected to {} ({})", args[1], target_addr);

    let stdin = BufReader::new(stdin());
    for line in stdin.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line == "/quit" { break; }
        if line.is_empty() { continue; }

        if let Err(_) = sock.send_to(line.as_bytes(), target_addr) {
            eprintln!("Send failed");
        }
    }

    running.store(false, Ordering::SeqCst);
    drop(sock); // close
    recv.join().ok();

    0
}
