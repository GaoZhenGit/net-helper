use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Pipe {
    rx: Mutex<mpsc::Receiver<String>>,
    eof: AtomicBool,
}

impl Pipe {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let stdin = BufReader::new(stdin());
            for line in stdin.lines() {
                match line {
                    Ok(l) => { if tx.send(l).is_err() { break; } }
                    Err(_) => break,
                }
            }
        });
        Self { rx: Mutex::new(rx), eof: AtomicBool::new(false) }
    }

    pub fn cleanup(&self) {}

    pub fn poll(&self, timeout: Duration) -> Option<String> {
        match self.rx.lock().unwrap().recv_timeout(timeout) {
            Ok(line) => Some(line),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.eof.store(true, Ordering::SeqCst);
                None
            }
        }
    }

    pub fn is_eof(&self) -> bool { self.eof.load(Ordering::SeqCst) }

    pub fn write(&self, f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
        let mut o = stdout();
        f(&mut o).unwrap_or(());
        let _ = o.flush();
    }
}
