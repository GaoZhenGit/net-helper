mod console;
mod net;
mod tls;
mod udp;
mod tcp;
mod dns;
mod version;

fn print_usage() {
    println!("net-helper - network diagnostic tool\n\
         \nUsage:\n  \
         net-helper -u  <ip|domain> <port>   UDP send/receive\n  \
         net-helper -t  <ip|domain> <port>   TCP connect\n  \
         net-helper -t  -tls <ip|domain> <port>  TCP with TLS\n  \
         net-helper -d  <domain>             DNS lookup\n  \
         net-helper -v, --version            Show version\n  \
         net-helper -h, --help               Show this help");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let flag = args[1].as_str();
    let sub = &args[1..];

    if flag == "-h" || flag == "--help" {
        print_usage();
        std::process::exit(0);
    }
    if flag == "-v" || flag == "--version" {
        version::print();
        std::process::exit(0);
    }
    if flag == "-d" || flag == "--dns" {
        std::process::exit(dns::run(sub));
    }

    console::init();

    let code = match flag {
        "-u" | "--udp" => udp::run(sub),
        "-t" | "--tcp" => tcp::run(sub),
        _ => {
            eprintln!("Unknown flag: {}", flag);
            print_usage();
            1
        }
    };

    console::cleanup();
    std::process::exit(code);
}
