use std::io::{stdout, Write};
use std::sync::Mutex;

pub(crate) static OUT: Mutex<()> = Mutex::new(());

/// Atomically write to stdout (mutex-protected), then flush.
pub(crate) fn put(f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
    let _lock = OUT.lock().unwrap();
    let mut o = stdout();
    f(&mut o).unwrap();
    o.flush().unwrap();
}

/// Clear previous line: move up, CR, erase (sender: erases user input).
pub(crate) fn clr_up() -> &'static str {
    "\x1b[1A\r\x1b[2K"
}

/// Clear current line: CR, erase (receiver: erases `>` prompt).
pub(crate) fn clr() -> &'static str {
    "\r\x1b[2K"
}

pub(crate) fn size_fmt(n: usize) -> String {
    if n < 1024 {
        format!("{}b", n)
    } else if n < 1024 * 1024 {
        format!("{:.1}KB", n as f64 / 1024.0)
    } else {
        format!("{:.2}MB", n as f64 / (1024.0 * 1024.0))
    }
}

/// Write data line-by-line with a per-line prefix.
pub(crate) fn write_prefixed(o: &mut dyn Write, data: &[u8], prefix: &str) -> std::io::Result<()> {
    let pfx = prefix.as_bytes();
    let mut start = 0;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            o.write_all(pfx)?;
            o.write_all(&data[start..i])?;
            o.write_all(b"\n")?;
            start = i + 1;
        }
    }
    if start < data.len() {
        o.write_all(pfx)?;
        o.write_all(&data[start..])?;
        o.write_all(b"\n")?;
    }
    Ok(())
}
