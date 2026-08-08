use std::net::IpAddr;

pub fn rate_limit_key(ip_addr: IpAddr, method: &str, path: &str) -> String {
    format!("{}:{}:{}", ip_addr, method, path)
}
