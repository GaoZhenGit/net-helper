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

pub struct Term {
    state: Mutex<InputState>,
}

struct InputState {
    buf: String,
    pos: usize,
}

impl Term {
    pub fn new() -> Self {
        let _ = terminal::enable_raw_mode();
        let _ = execute!(stdout(), crossterm::event::EnableBracketedPaste);
        Self { state: Mutex::new(InputState { buf: String::new(), pos: 0 }) }
    }

    pub fn cleanup(&self) {
        self.clear_prompt();
        let _ = execute!(stdout(), crossterm::event::DisableBracketedPaste);
        let _ = terminal::disable_raw_mode();
    }

    pub fn poll(&self, timeout: Duration) -> Option<String> {
        if !event::poll(timeout).unwrap_or(false) { return None; }
        match event::read().unwrap() {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    super::set_quit();
                    return None;
                }
                let mut s = self.state.lock().unwrap();
                match key.code {
                    KeyCode::Enter => {
                        let line = std::mem::take(&mut s.buf);
                        s.pos = 0;
                        self.clear_prompt();
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
                self.redraw_prompt(&s);
            }
            Event::Paste(data) => {
                let mut s = self.state.lock().unwrap();
                let p = s.pos;
                s.buf.insert_str(p, &data);
                s.pos = p + data.len();
                self.redraw_prompt(&s);
            }
            Event::Resize(_, _) => {
                let s = self.state.lock().unwrap();
                self.redraw_prompt(&s);
            }
            _ => {}
        }
        None
    }

    pub fn cursor_state(&self) -> (String, usize) {
        let s = self.state.lock().unwrap();
        (s.buf.clone(), s.pos)
    }

    pub fn write(&self, f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
        let (prompt, pos) = self.cursor_state();
        let mut o = stdout();
        // 清除当前提示行，输出内容（可能多行），在输出结束后重绘提示
        let _ = execute!(o, Clear(ClearType::CurrentLine), cursor::MoveToColumn(0));
        f(&mut o).unwrap_or(());
        let _ = o.flush();
        let _ = execute!(o, Print(format!("\r{}> {}", Clear(ClearType::CurrentLine), prompt)));
        let col = (2 + pos) as u16;
        let _ = execute!(o, cursor::MoveToColumn(col));
        let _ = o.flush();
    }

    fn clear_prompt(&self) {
        let _ = execute!(stdout(), Clear(ClearType::CurrentLine), cursor::MoveToColumn(0));
    }
    fn redraw_prompt(&self, s: &InputState) {
        let _ = execute!(stdout(),
            Clear(ClearType::CurrentLine), cursor::MoveToColumn(0),
            Print(format!("> {}", s.buf)),
        );
        let col = (2 + s.pos) as u16;
        let _ = execute!(stdout(), cursor::MoveToColumn(col));
    }
}
