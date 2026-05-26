mod input;
mod output;
mod udp;
mod tcp;
mod dns;
mod version;

pub(crate) use output::{clr, clr_up, put, size_fmt, write_prefixed};

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
    let sub = &args[1..];

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
