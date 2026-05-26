use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

/// Spawn a receiver thread; returns the close-signal receiver.
pub fn spawn_receiver<R: Read + Send + 'static>(
    mut stream: R,
    running: &Arc<AtomicBool>,
) -> mpsc::Receiver<()> {
    let r = running.clone();
    let (close_tx, close_rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    crate::console::status("Connection closed by remote");
                    r.store(false, Ordering::SeqCst);
                    let _ = close_tx.send(());
                    break;
                }
                Ok(n) => crate::console::recv(&buf[..n]),
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if !r.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }
                Err(_) => {
                    crate::console::status("Connection lost");
                    r.store(false, Ordering::SeqCst);
                    let _ = close_tx.send(());
                    break;
                }
            }
        }
    });
    close_rx
}

/// Main interactive loop: reads user input, sends via `do_write`, checks for close.
pub fn interactive<W: Write>(
    mut stream: W,
    running: Arc<AtomicBool>,
    close_rx: mpsc::Receiver<()>,
) -> i32 {
    loop {
        if crate::console::quit_requested() || crate::console::eof() {
            break;
        }
        if let Some(line) = crate::console::poll(Duration::from_millis(200)) {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            if line == "/quit" {
                break;
            }
            let mut data = line.into_bytes();
            data.push(b'\n');
            if stream.write_all(&data).is_err() || stream.flush().is_err() {
                crate::console::status("Send failed");
                running.store(false, Ordering::SeqCst);
                break;
            }
            crate::console::send(&data);
        }
        if close_rx.try_recv().is_ok() || !running.load(Ordering::SeqCst) {
            break;
        }
    }
    // Wait for receiver to flush remaining data (or remote close)
    let _ = close_rx.recv_timeout(Duration::from_secs(5));
    running.store(false, Ordering::SeqCst);
    drop(stream);
    0
}

/// Connect with a timeout via a spawned thread + channel.
pub fn connect_timeout(addr: std::net::SocketAddr, timeout: Duration) -> std::io::Result<TcpStream> {
    let (tx, rx) = mpsc::channel();
    let a = addr;
    thread::spawn(move || { let _ = tx.send(TcpStream::connect(a)); });
    rx.recv_timeout(timeout)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "connect timed out"))
        .and_then(|r| r)
}
