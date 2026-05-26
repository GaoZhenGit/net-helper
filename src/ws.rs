use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use tungstenite::Message;

pub fn run(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: net-helper -ws <ws|wss://host[:port][/path]>");
        return 1;
    }

    let url = &args[1];
    if !url.starts_with("ws://") && !url.starts_with("wss://") {
        eprintln!("URL must start with ws:// or wss://");
        return 1;
    }

    let (mut ws, _) = match tungstenite::connect(url) {
        Ok(w) => w,
        Err(e) => { eprintln!("{}", e); return 1; }
    };

    // Set non-blocking so read() does not hold the mutex forever
    match ws.get_mut() {
        tungstenite::stream::MaybeTlsStream::Plain(s) => { s.set_nonblocking(true).ok(); }
        tungstenite::stream::MaybeTlsStream::Rustls(s) => {
            s.sock.set_nonblocking(true).ok();
        }
        _ => {}
    }

    crate::console::println(&format!("Connected to {}", url));

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let (close_tx, close_rx) = mpsc::channel::<()>();
    let ws = Arc::new(Mutex::new(ws));

    let ws_rx = ws.clone();
    thread::spawn(move || {
        loop {
            let result = ws_rx.lock().unwrap().read();
            match result {
                Ok(Message::Text(t))   => crate::console::recv(t.as_bytes()),
                Ok(Message::Binary(b)) => crate::console::recv(&b),
                Ok(Message::Close(_)) => {
                    if r.load(Ordering::SeqCst) {
                        crate::console::status("Connection closed by remote");
                    }
                    r.store(false, Ordering::SeqCst);
                    let _ = close_tx.send(());
                    break;
                }
                Err(tungstenite::error::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if !r.load(Ordering::SeqCst) { let _ = close_tx.send(()); break; }
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }
                Err(_) => {
                    if r.load(Ordering::SeqCst) {
                        crate::console::status("Connection lost");
                    }
                    r.store(false, Ordering::SeqCst);
                    let _ = close_tx.send(());
                    break;
                }
                _ => {} // Ping/Pong handled by tungstenite
            }
        }
    });

    let ws_tx = ws.clone();
    loop {
        if crate::console::quit_requested() || crate::console::eof() { break; }
        if let Some(line) = crate::console::poll(Duration::from_millis(200)) {
            if !running.load(Ordering::SeqCst) { break; }
            if line == "/quit" { break; }
            if line.is_empty() { continue; }
            if ws_tx.lock().unwrap().send(Message::Text(line.clone())).is_err() {
                crate::console::status("Send failed");
                running.store(false, Ordering::SeqCst);
                break;
            }
            crate::console::send(line.as_bytes());
        }
        if close_rx.try_recv().is_ok() || !running.load(Ordering::SeqCst) { break; }
    }

    let exited = crate::console::quit_requested();
    running.store(false, Ordering::SeqCst);
    // Try to send close frame (non-blocking, skip if receiver holds lock)
    if let Ok(mut w) = ws.try_lock() { let _ = w.close(None); }
    let _ = close_rx.recv_timeout(Duration::from_millis(500));
    if exited { crate::console::status("Closed by user"); }
    drop(ws);
    0
}
