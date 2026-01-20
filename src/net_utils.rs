// sentiric-rtp-core/src/net_utils.rs

use std::net::IpAddr;

pub fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 10.0.0.0/8
            if octets[0] == 10 { return true; }
            // 172.16.0.0/12
            if octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31) { return true; }
            // 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 { return true; }
            // 127.0.0.0/8 (Loopback)
            if octets[0] == 127 { return true; }
            false
        }
        IpAddr::V6(ipv6) => {
            // Loopback (::1)
            if ipv6.is_loopback() { return true; }
            // Unique Local (fc00::/7)
            if (ipv6.segments()[0] & 0xfe00) == 0xfc00 { return true; }
            // Link Local (fe80::/10)
            if (ipv6.segments()[0] & 0xffc0) == 0xfe80 { return true; }
            false
        }
    }
}

pub fn is_public_ip(ip: IpAddr) -> bool {
    !is_private_ip(ip)
}