mod term;
mod pipe;

use std::io::{IsTerminal, Write};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static QUIT: AtomicBool = AtomicBool::new(false);
static BACKEND: OnceLock<Backend> = OnceLock::new();

enum Backend {
    Raw(term::Term),
    Line(pipe::Pipe),
}

fn get() -> &'static Backend {
    BACKEND.get_or_init(|| {
        if std::io::stdout().is_terminal() {
            Backend::Raw(term::Term::new())
        } else {
            Backend::Line(pipe::Pipe::new())
        }
    })
}

pub fn init()          { get(); }
pub fn cleanup()       { match get() { Backend::Raw(r) => r.cleanup(), Backend::Line(l) => l.cleanup() } }
pub fn quit_requested() -> bool { QUIT.load(Ordering::SeqCst) }
pub fn eof() -> bool {
    match get() { Backend::Line(l) => l.is_eof(), Backend::Raw(_) => false }
}
pub(crate) fn set_quit()       { QUIT.store(true, Ordering::SeqCst); }

pub fn poll(timeout: Duration) -> Option<String> {
    match get() { Backend::Raw(r) => r.poll(timeout), Backend::Line(l) => l.poll(timeout) }
}

pub fn write(f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
    match get() { Backend::Raw(r) => r.write(f), Backend::Line(l) => l.write(f) }
}

pub fn println(s: &str)  { write(|o| { write!(o, "{}{}", s, eol())?; Ok(()) }); }
pub fn status(msg: &str) { write(|o| { write!(o, "{}{}", msg, eol())?; Ok(()) }); }

fn eol() -> &'static str {
    match get() {
        Backend::Raw(_) => "\r\n",
        Backend::Line(_) => "\n",
    }
}

pub fn recv(data: &[u8]) {
    write(|o| { write!(o, "[recv {}]{}", size_fmt(data.len()), eol())?; write_prefixed(o, data, "<- ") });
}

pub fn recv_from(from: &str, data: &[u8]) {
    write(|o| { write!(o, "[recv {} {}]{}", from, size_fmt(data.len()), eol())?; write_prefixed(o, data, "<- ") });
}

pub fn send(data: &[u8]) {
    write(|o| { write!(o, "[send {}]{}", size_fmt(data.len()), eol())?; write_prefixed(o, data, "-> ") });
}

// ── shared ─────────────────────────────────────────────

fn size_fmt(n: usize) -> String {
    if n < 1024 { format!("{}b", n) }
    else if n < 1048576 { format!("{:.1}KB", n as f64 / 1024.0) }
    else { format!("{:.2}MB", n as f64 / 1048576.0) }
}

fn write_prefixed(o: &mut dyn Write, data: &[u8], prefix: &str) -> std::io::Result<()> {
    let pfx = prefix.as_bytes();
    let le = eol().as_bytes();
    let mut s = 0;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' { o.write_all(pfx)?; o.write_all(&data[s..i])?; o.write_all(le)?; s = i + 1; }
    }
    if s < data.len() { o.write_all(pfx)?; o.write_all(&data[s..])?; o.write_all(le)?; }
    Ok(())
}
