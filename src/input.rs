use std::io::{stdin, BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

/// Spawn a background thread that reads stdin line-by-line.
/// Returns the receiver end of the channel.
pub fn reader() -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let stdin = BufReader::new(stdin());
        for line in stdin.lines() {
            match line {
                Ok(l) => {
                    if tx.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    rx
}
