mod console;
mod udp;
mod tcp;
mod dns;
mod version;

fn print_usage() {
    println!("net-helper - network diagnostic tool\n\
         \nUsage:\n  \
         net-helper -u <ip|domain> <port>   UDP send/receive\n  \
         net-helper -t <ip|domain> <port>   TCP connect\n  \
         net-helper -d <domain>              DNS lookup\n  \
         net-helper -v, --version            Show version");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let flag = args[1].as_str();
    let sub = &args[1..];

    // DNS / version don't need raw terminal
    if flag == "-d" || flag == "--dns" {
        std::process::exit(dns::run(sub));
    }
    if flag == "-v" || flag == "--version" {
        version::print();
        std::process::exit(0);
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
