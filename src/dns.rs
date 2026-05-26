use std::net::ToSocketAddrs;

pub fn run(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: net-helper -d <domain>");
        return 1;
    }

    let domain = &args[1];
    let addr_str = format!("{}:0", domain);

    let addrs = match addr_str.to_socket_addrs() {
        Ok(iter) => {
            let v: Vec<_> = iter.collect();
            if v.is_empty() {
                eprintln!("Failed to resolve: {}", domain);
                return 1;
            }
            v
        }
        Err(_) => {
            eprintln!("Failed to resolve: {}", domain);
            return 1;
        }
    };

    println!(
        "{} ({} record{}):",
        domain,
        addrs.len(),
        if addrs.len() > 1 { "s" } else { "" }
    );

    for addr in &addrs {
        let ip = addr.ip();
        if ip.is_ipv4() {
            println!("  IPv4  {}", ip);
        } else {
            println!("  IPv6  {}", ip);
        }
    }

    0
}
