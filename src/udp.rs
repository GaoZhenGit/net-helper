use std::net::{UdpSocket, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use crate::{clr, clr_up, put, size_fmt, write_prefixed};

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

    sock.set_read_timeout(Some(Duration::from_millis(500)))
        .ok();

    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let rx = crate::input::reader();

    let rx_sock = sock.try_clone().expect("UdpSocket::try_clone failed");
    thread::spawn(move || {
        let mut buf = [0u8; 65536];
        while r.load(Ordering::SeqCst) {
            match rx_sock.recv_from(&mut buf) {
                Ok((n, from)) => {
                    put(|o| {
                        write!(o, "{}[recv {} {}]", clr(), from, size_fmt(n))?;
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
                    if !r.load(Ordering::SeqCst) {
                        break;
                    }
                    continue;
                }
            }
        }
    });

    println!("UDP connected to {} ({})", args[1], target_addr);

    put(|o| write!(o, "> "));
    loop {
        match rx.recv() {
            Ok(line) => {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                if line == "/quit" {
                    break;
                }
                if line.is_empty() {
                    continue;
                }
                let data = line.as_bytes();
                if sock.send_to(data, target_addr).is_err() {
                    eprintln!("Send failed");
                } else {
                    let len = data.len();
                    put(|o| {
                        write!(o, "{}[send {}]", clr_up(), size_fmt(len))?;
                        o.write_all(b"\n")?;
                        write_prefixed(o, data, "-> ")?;
                        write!(o, "> ")
                    });
                }
            }
            Err(_) => break,
        }
    }

    running.store(false, Ordering::SeqCst);
    drop(sock);
    0
}
