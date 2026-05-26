use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub fn run(args: &[String]) -> i32 {
    if args.len() < 3 {
        eprintln!("Usage: net-helper -u <ip|domain> <port>");
        return 1;
    }

    let port: u16 = args[2].parse().unwrap_or(0);
    let ipv6 = args.iter().any(|a| a == "-ipv6" || a == "-6");
    let addrs = crate::dns::resolve(&args[1], port, ipv6);
    let target_addr = match addrs.first() {
        Some(a) => *a,
        None => { eprintln!("Failed to resolve: {}", args[1]); return 1; }
    };

    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => { eprintln!("Failed to create UDP socket"); return 1; }
    };
    sock.set_read_timeout(Some(Duration::from_millis(500))).ok();

    let running = std::sync::Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let rx_sock = sock.try_clone().expect("UdpSocket::try_clone failed");
    thread::spawn(move || {
        let mut buf = [0u8; 65536];
        while r.load(Ordering::SeqCst) {
            match rx_sock.recv_from(&mut buf) {
                Ok((n, from)) => crate::console::recv_from(&from.to_string(), &buf[..n]),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                          || e.kind() == std::io::ErrorKind::TimedOut => {
                    if !r.load(Ordering::SeqCst) { break; } continue;
                }
                Err(_) => { if !r.load(Ordering::SeqCst) { break; } continue; }
            }
        }
    });

    crate::console::println(&format!("UDP connected to {} ({})", args[1], target_addr));

    loop {
        if crate::console::quit_requested() || crate::console::eof() { break; }
        if let Some(line) = crate::console::poll(Duration::from_millis(200)) {
            if !running.load(Ordering::SeqCst) { break; }
            if line == "/quit" { break; }
            if line.is_empty() { continue; }
            if sock.send_to(line.as_bytes(), target_addr).is_err() {
                eprintln!("Send failed");
            } else {
                crate::console::send(line.as_bytes());
            }
        }
        if !running.load(Ordering::SeqCst) { break; }
    }

    running.store(false, Ordering::SeqCst);
    drop(sock);
    thread::sleep(Duration::from_millis(600));  // let receiver flush
    0
}
