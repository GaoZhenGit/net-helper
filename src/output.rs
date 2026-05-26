use std::io::{stdout, Write};
use std::sync::Mutex;

static OUT: Mutex<()> = Mutex::new(());

fn lock(f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
    let _guard = OUT.lock().unwrap();
    let mut o = stdout();
    f(&mut o).unwrap();
    o.flush().unwrap();
}

// --- low-level ANSI helpers ---

fn clr_up() -> &'static str { "\x1b[1A\r\x1b[2K" }  // clear previous line
fn clr()    -> &'static str { "\r\x1b[2K" }          // clear current line

// --- size formatting ---

fn size_fmt(n: usize) -> String {
    if n < 1024 {
        format!("{}b", n)
    } else if n < 1024 * 1024 {
        format!("{:.1}KB", n as f64 / 1024.0)
    } else {
        format!("{:.2}MB", n as f64 / (1024.0 * 1024.0))
    }
}

// --- content output ---

pub(crate) fn status(msg: &str) {
    let s = msg.to_string();
    lock(move |o| writeln!(o, "{}{s}", clr()))
}

pub(crate) fn prompt() {
    lock(|o| write!(o, "> "))
}

pub(crate) fn send(data: &[u8]) {
    let len = data.len();
    lock(move |o| {
        write!(o, "{}[send {}]\n", clr_up(), size_fmt(len))?;
        write_prefixed(o, data, "-> ")?;
        write!(o, "> ")
    })
}

pub(crate) fn recv(data: &[u8]) {
    let len = data.len();
    lock(move |o| {
        write!(o, "{}[recv {}]\n", clr(), size_fmt(len))?;
        write_prefixed(o, data, "<- ")?;
        write!(o, "> ")
    })
}

pub(crate) fn recv_from(from: &str, data: &[u8]) {
    let n = data.len();
    let addr = from.to_string();
    lock(move |o| {
        write!(o, "{}[recv {} {}]\n", clr(), addr, size_fmt(n))?;
        write_prefixed(o, data, "<- ")?;
        write!(o, "> ")
    })
}

// --- internal ---

fn write_prefixed(o: &mut dyn Write, data: &[u8], prefix: &str) -> std::io::Result<()> {
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
