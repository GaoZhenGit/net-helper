use std::io::{stdout, Write};
use std::sync::Mutex;
use std::time::Duration;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};

struct InputState {
    buf: String,
    pos: usize,
}

static STATE: Mutex<InputState> = Mutex::new(InputState { buf: String::new(), pos: 0 });
static OUT: Mutex<()> = Mutex::new(());
static QUIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn quit_requested() -> bool { QUIT.load(std::sync::atomic::Ordering::SeqCst) }

pub fn init() {
    let _ = terminal::enable_raw_mode();
    let _ = execute!(stdout(), crossterm::event::EnableBracketedPaste);
    redraw();
}

pub fn cleanup() {
    clear_prompt();
    let _ = execute!(stdout(), crossterm::event::DisableBracketedPaste);
    let _ = terminal::disable_raw_mode();
}

/// Process keyboard events for up to `timeout`. Returns completed line on Enter.
pub fn poll(timeout: Duration) -> Option<String> {
    if !event::poll(timeout).unwrap_or(false) {
        return None;
    }
    match event::read().unwrap() {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
            // Ctrl+C → quit
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                QUIT.store(true, std::sync::atomic::Ordering::SeqCst);
                return None;
            }
            let mut s = STATE.lock().unwrap();
            match key.code {
                KeyCode::Enter => {
                    let line = std::mem::take(&mut s.buf);
                    s.pos = 0;
                    clear_prompt();
                    return Some(line);
                }
                KeyCode::Backspace => { let p = s.pos; if p > 0 { s.buf.remove(p - 1); s.pos = p - 1; } }
                KeyCode::Delete    => { let p = s.pos; if p < s.buf.len() { s.buf.remove(p); } }
                KeyCode::Left      => { if s.pos > 0 { s.pos -= 1; } }
                KeyCode::Right     => { if s.pos < s.buf.len() { s.pos += 1; } }
                KeyCode::Home      => { s.pos = 0; }
                KeyCode::End       => { s.pos = s.buf.len(); }
                KeyCode::Char(c)   => { let p = s.pos; s.buf.insert(p, c); s.pos = p + 1; }
                _ => {}
            }
            redraw_prompt(&s);
        }
        Event::Paste(data) => {
            let mut s = STATE.lock().unwrap();
            let p = s.pos;
            s.buf.insert_str(p, &data);
            s.pos = p + data.len();
            redraw_prompt(&s);
        }
        Event::Resize(_, _) => {
            let s = STATE.lock().unwrap();
            redraw_prompt(&s);
        }
        _ => {}
    }
    None
}

// ── output (locked, restores prompt after) ────────────

pub fn write(f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
    let _lock = OUT.lock().unwrap();
    let prompt = STATE.lock().unwrap().buf.clone();

    let mut o = stdout();
    let _ = execute!(o, Clear(ClearType::CurrentLine), cursor::MoveToColumn(0));
    let _ = writeln!(o);
    let _ = execute!(o, cursor::MoveUp(1), cursor::MoveToColumn(0));
    f(&mut o).unwrap_or(());
    let _ = o.flush();

    let _ = execute!(o, Print(format!("\r{}> {}", Clear(ClearType::CurrentLine), prompt)));
    let _ = o.flush();
}

pub fn println(s: &str)       { write(|o| { writeln!(o, "{s}")?; Ok(()) }); }
pub fn status(msg: &str)      { write(|o| { writeln!(o, "{msg}")?; Ok(()) }); }

pub fn recv(data: &[u8]) {
    write(|o| {
        write!(o, "[recv {}]\n", size_fmt(data.len()))?;
        write_prefixed(o, data, "<- ")
    });
}

pub fn recv_from(from: &str, data: &[u8]) {
    write(|o| {
        write!(o, "[recv {} {}]\n", from, size_fmt(data.len()))?;
        write_prefixed(o, data, "<- ")
    });
}

pub fn send(data: &[u8]) {
    write(|o| {
        write!(o, "[send {}]\n", size_fmt(data.len()))?;
        write_prefixed(o, data, "-> ")
    });
}

// ── internal ──────────────────────────────────────────

fn size_fmt(n: usize) -> String {
    if n < 1024 { format!("{}b", n) }
    else if n < 1048576 { format!("{:.1}KB", n as f64 / 1024.0) }
    else { format!("{:.2}MB", n as f64 / 1048576.0) }
}

fn write_prefixed(o: &mut dyn Write, data: &[u8], prefix: &str) -> std::io::Result<()> {
    let pfx = prefix.as_bytes();
    let mut s = 0;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            o.write_all(pfx)?; o.write_all(&data[s..i])?; o.write_all(b"\n")?; s = i + 1;
        }
    }
    if s < data.len() { o.write_all(pfx)?; o.write_all(&data[s..])?; o.write_all(b"\n")?; }
    Ok(())
}

fn clear_prompt() {
    let _ = execute!(stdout(), Clear(ClearType::CurrentLine), cursor::MoveToColumn(0));
}
fn redraw() {
    let s = STATE.lock().unwrap();
    redraw_prompt(&s);
}
fn redraw_prompt(s: &InputState) {
    let _ = execute!(stdout(),
        Clear(ClearType::CurrentLine), cursor::MoveToColumn(0),
        Print(format!("> {}", s.buf)),
    );
    let col = (2 + s.pos) as u16;
    let _ = execute!(stdout(), cursor::MoveToColumn(col));
}
