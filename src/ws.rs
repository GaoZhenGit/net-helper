use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier, HandshakeSignatureValid};
use rustls::DigitallySignedStruct;
use tungstenite::{Message, client::IntoClientRequest};

// ── helpers ──────────────────────────────────────────────

fn loop_addrs(addrs: &[std::net::SocketAddr]) -> Option<TcpStream> {
    for addr in addrs { if let Ok(s) = crate::net::connect_timeout(*addr, Duration::from_secs(2)) { return Some(s); } }
    None
}

// ── Io trait — unify blocking/nonblocking across TcpStream and TLS ─

trait Io: Read + Write + Send { fn set_nonblocking(&self, on: bool) -> std::io::Result<()>; }
impl Io for TcpStream {
    fn set_nonblocking(&self, on: bool) -> std::io::Result<()> { self.set_nonblocking(on) }
}
impl Io for rustls::StreamOwned<rustls::ClientConnection, TcpStream> {
    fn set_nonblocking(&self, on: bool) -> std::io::Result<()> { self.sock.set_nonblocking(on) }
}
type Ws = tungstenite::WebSocket<Box<dyn Io>>;

// ── TLS config ───────────────────────────────────────────

fn make_config(permissive: bool) -> Arc<rustls::ClientConfig> {
    if permissive {
        #[derive(Debug)]
        struct Permissive { provider: &'static rustls::crypto::CryptoProvider }
        impl ServerCertVerifier for Permissive {
            fn verify_server_cert(&self, _ee: &CertificateDer<'_>, _chain: &[CertificateDer<'_>], _sn: &ServerName<'_>, _ocsp: &[u8], _now: UnixTime) -> Result<ServerCertVerified, rustls::Error> {
                Ok(ServerCertVerified::assertion())
            }
            fn verify_tls12_signature(&self, m: &[u8], c: &CertificateDer<'_>, d: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
                rustls::crypto::verify_tls12_signature(m, c, d, &self.provider.signature_verification_algorithms)
            }
            fn verify_tls13_signature(&self, m: &[u8], c: &CertificateDer<'_>, d: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
                rustls::crypto::verify_tls13_signature(m, c, d, &self.provider.signature_verification_algorithms)
            }
            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                self.provider.signature_verification_algorithms.supported_schemes()
            }
        }
        let p: &'static _ = Box::leak(Box::new(rustls::crypto::ring::default_provider()));
        Arc::new(
            rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
                .dangerous().with_custom_certificate_verifier(Arc::new(Permissive { provider: p })).with_no_client_auth()
        )
    } else {
        let mut roots = rustls::RootCertStore::empty();
        for a in webpki_roots::TLS_SERVER_ROOTS { roots.roots.push(a.to_owned()); }
        Arc::new(
            rustls::ClientConfig::builder_with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
                .with_root_certificates(roots).with_no_client_auth()
        )
    }
}

fn try_handshake(sock: &TcpStream, domain: &str, config: Arc<rustls::ClientConfig>) -> Result<rustls::StreamOwned<rustls::ClientConnection, TcpStream>, String> {
    let sn: ServerName<'_> = domain.to_string().try_into().map_err(|_| "bad domain".to_string())?;
    let conn = rustls::ClientConnection::new(config, sn).map_err(|e| e.to_string())?;
    let sock_clone = sock.try_clone().map_err(|e| e.to_string())?;
    sock_clone.set_read_timeout(Some(Duration::from_secs(10))).map_err(|e| e.to_string())?;
    let mut stream = rustls::StreamOwned::new(conn, sock_clone);
    loop {
        match stream.conn.complete_io(&mut stream.sock) {
            Ok((_, 0)) if stream.conn.is_handshaking() => continue,
            Ok(_) => break,
            Err(e) => return Err(format!("{}", e)),
        }
    }
    stream.sock.set_read_timeout(Some(Duration::from_secs(10))).ok();
    Ok(stream)
}

/// TLS connect — tries default verifier, falls back to permissive with reconnect.
fn tls_connect(sock: TcpStream, domain: &str, addrs: &[std::net::SocketAddr]) -> Result<Box<dyn Io>, String> {
    // Try with cert verification
    match try_handshake(&sock, domain, make_config(false)) {
        Ok(stream) => return Ok(Box::new(stream)),
        Err(e) => eprintln!("{}", e),
    }
    // Fallback: reconnect + permissive
    let raw2 = loop_addrs(addrs).ok_or("Failed to reconnect")?;
    eprintln!("Warning: certificate verification disabled for '{}'", domain);
    try_handshake(&raw2, domain, make_config(true)).map(|s| Box::new(s) as Box<dyn Io>)
}

// ── entry ────────────────────────────────────────────────

pub fn run(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: net-helper -ws <ws|wss://host[:port][/path]>");
        return 1;
    }
    let url_str = &args[1];
    let is_ssl = url_str.starts_with("wss://");
    if !url_str.starts_with("ws://") && !is_ssl {
        eprintln!("URL must start with ws:// or wss://"); return 1;
    }

    // Parse URL
    let rest = &url_str[if is_ssl { 6 } else { 5 }..];
    let (host_port, path) = match rest.find('/') { Some(i) => (&rest[..i], &rest[i..]), None => (rest, "/") };
    let (host, port_str) = match host_port.find(':') { Some(i) => (&host_port[..i], &host_port[i+1..]), None => (host_port, if is_ssl {"443"} else {"80"}) };
    let port: u16 = port_str.parse().unwrap_or(if is_ssl {443} else {80});

    // Resolve + connect
    let ipv6 = args.iter().any(|a| a == "-ipv6" || a == "-6");
    let addrs = crate::dns::resolve(host, port, ipv6);
    if addrs.is_empty() { eprintln!("Failed to resolve: {}", host); return 1; }
    let raw = match loop_addrs(&addrs) { Some(s) => s, None => { eprintln!("Failed to connect to {}:{}", host, port); return 1; } };
    let peer = raw.peer_addr().map(|a| a.to_string()).unwrap_or_default();

    // WS or WSS handshake — same shape, only stream construction differs
    let stream: Box<dyn Io> = if is_ssl {
        match tls_connect(raw, host, &addrs) { Ok(s) => s, Err(e) => { eprintln!("{}", e); return 1; } }
    } else {
        Box::new(raw)
    };
    let handshake_url = format!("{}://{}:{}{}", if is_ssl {"wss"} else {"ws"}, host, port, path);
    let req = match handshake_url.as_str().into_client_request() {
        Ok(r) => r,
        Err(e) => { eprintln!("Invalid URL: {}", e); return 1; }
    };
    let mut ws: Ws = match tungstenite::client(req, stream) { Ok((w,_)) => w, Err(e) => { eprintln!("{}", e); return 1; } };

    // Interactive loop
    ws.get_mut().set_nonblocking(true).ok();
    crate::console::println(&format!("Connected to {} ({})", url_str, peer));

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let (close_tx, close_rx) = mpsc::channel::<()>();
    let ws = Arc::new(Mutex::new(ws));

    let ws_rx = ws.clone();
    thread::spawn(move || loop {
        let result = ws_rx.lock().unwrap().read();
        match result {
            Ok(Message::Text(t))   => crate::console::recv(t.as_bytes()),
            Ok(Message::Binary(b)) => crate::console::recv(&b),
            Ok(Message::Close(_)) => {
                if r.load(Ordering::SeqCst) { crate::console::status("Connection closed by remote"); }
                r.store(false, Ordering::SeqCst); let _ = close_tx.send(()); break;
            }
            Err(tungstenite::error::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if !r.load(Ordering::SeqCst) { let _ = close_tx.send(()); break; }
                thread::sleep(Duration::from_millis(20)); continue;
            }
            Err(_) => {
                if r.load(Ordering::SeqCst) { crate::console::status("Connection lost"); }
                r.store(false, Ordering::SeqCst); let _ = close_tx.send(()); break;
            }
            _ => {}
        }
    });

    let ws_tx = ws.clone();
    let mut eof = false;
    loop {
        if crate::console::quit_requested() { break; }
        if !eof && crate::console::eof() { eof = true; }
        if let Some(line) = crate::console::poll(Duration::from_millis(200)) {
            if !running.load(Ordering::SeqCst) { break; }
            if line == "/quit" { break; }
            if line.is_empty() { continue; }
            if ws_tx.lock().unwrap().send(Message::Text(line.clone())).is_err() {
                crate::console::status("Send failed"); running.store(false, Ordering::SeqCst); break;
            }
            crate::console::send(line.as_bytes());
        }
        if close_rx.try_recv().is_ok() || !running.load(Ordering::SeqCst) { break; }
        if eof && close_rx.try_recv().is_ok() { break; }
        if eof { let _ = close_rx.recv_timeout(Duration::from_secs(3)); break; }
    }

    let exited = crate::console::quit_requested();
    let wait = if exited { 500 } else { 3000 };
    running.store(false, Ordering::SeqCst);
    if let Ok(mut w) = ws.try_lock() { let _ = w.close(None); }
    let _ = close_rx.recv_timeout(Duration::from_millis(wait));
    if exited { crate::console::status("Closed by user"); }
    drop(ws);
    0
}
