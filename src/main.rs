mod udp;
mod tcp;
mod dns;
mod version;

use std::io::{stdout, Write};
use std::sync::Mutex;

// Global lock for serialising multi-threaded console output
pub(crate) static OUT: Mutex<()> = Mutex::new(());

/// Write a complete message to stdout atomically (thread-safe), then flush.
pub(crate) fn put(f: impl FnOnce(&mut dyn Write) -> std::io::Result<()>) {
    let _lock = OUT.lock().unwrap();
    let mut o = stdout();
    f(&mut o).unwrap();
    o.flush().unwrap();
}

fn print_usage() {
    println!(
        "net-helper - network diagnostic tool\n\
         \nUsage:\n  \
         net-helper -u <ip|domain> <port>   UDP send/receive\n  \
         net-helper -t <ip|domain> <port>   TCP connect\n  \
         net-helper -d <domain>              DNS lookup\n  \
         net-helper -v, --version            Show version"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let flag = args[1].as_str();
    let sub = &args[1..]; // argv[0] = flag, argv[1..] = real args

    let code = match flag {
        "-u" | "--udp"     => udp::run(sub),
        "-t" | "--tcp"     => tcp::run(sub),
        "-d" | "--dns"     => dns::run(sub),
        "-v" | "--version" => { version::print(); 0 }
        _ => {
            eprintln!("Unknown flag: {}", flag);
            print_usage();
            1
        }
    };

    std::process::exit(code);
}
