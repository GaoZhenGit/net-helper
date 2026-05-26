use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use rustls::{ClientConnection, RootCertStore};

pub struct TlsStream {
    conn: ClientConnection,
    sock: TcpStream,
}

impl TlsStream {

    pub fn connect(mut sock: TcpStream, domain: &str) -> io::Result<Self> {
        let mut roots = RootCertStore::empty();
        roots.roots = webpki_roots::TLS_SERVER_ROOTS.iter().map(|a| a.to_owned()).collect();

        let config = rustls::ClientConfig::builder_with_protocol_versions(&[
                &rustls::version::TLS12,
                &rustls::version::TLS13,
            ])
            .with_root_certificates(roots)
            .with_no_client_auth();

        let server_name = domain
            .to_string()
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid domain"))?;

        let mut conn = ClientConnection::new(Arc::new(config), server_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Non-blocking handshake
        sock.set_nonblocking(true)?;
        loop {
            match conn.complete_io(&mut sock) {
                Ok((_, 0)) if conn.is_handshaking() => continue,
                Ok(_) => break,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        // Stay non-blocking for fast read/write
        let mut this = Self { conn, sock };
        // Drain any post-handshake data
        let _ = this.conn.complete_io(&mut this.sock);
        Ok(this)
    }

}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.conn.complete_io(&mut self.sock)?;
        self.conn.reader().read(buf)
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.conn.writer().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        // Keep trying complete_io until WouldBlock
        loop {
            match self.conn.complete_io(&mut self.sock) {
                Ok(_) => continue,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
