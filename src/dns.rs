use std::net::{SocketAddr, ToSocketAddrs};

pub fn resolve(host: &str, port: u16, ipv6: bool) -> Vec<SocketAddr> {
    let addrs: Vec<_> = match format!("{}:{}", host, port).to_socket_addrs() {
        Ok(i) => i.collect(),
        Err(_) => return vec![],
    };
    let is_ip_literal = host.parse::<std::net::IpAddr>().is_ok();
    if ipv6 || is_ip_literal { addrs } else { addrs.into_iter().filter(|a| a.ip().is_ipv4()).collect() }
}

pub fn run(args: &[String]) -> i32 {
    let positional: Vec<&str> = args.iter()
        .map(|s| s.as_str())
        .filter(|s| !s.starts_with('-'))
        .collect();
    if positional.len() < 1 {
        eprintln!("Usage: net-helper -d <domain>");
        return 1;
    }

    let domain = positional[0];
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
