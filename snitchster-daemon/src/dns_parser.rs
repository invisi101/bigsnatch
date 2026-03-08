use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Parsed DNS response containing domain -> IP mappings
pub struct DnsResponse {
    pub domain: String,
    pub addresses: Vec<(IpAddr, u32)>, // (IP, TTL)
}

/// Parse a raw DNS response packet and extract A/AAAA records.
pub fn parse_dns_response(data: &[u8]) -> Option<DnsResponse> {
    if data.len() < 12 {
        return None;
    }

    let flags = u16::from_be_bytes([data[2], data[3]]);
    let qr = (flags >> 15) & 1;
    if qr != 1 {
        return None;
    }

    let rcode = flags & 0x0F;
    if rcode != 0 {
        return None;
    }

    let qdcount = u16::from_be_bytes([data[4], data[5]]) as usize;
    let ancount = u16::from_be_bytes([data[6], data[7]]) as usize;

    if ancount == 0 {
        return None;
    }

    let mut offset = 12;

    // Parse question section
    let mut domain = String::new();
    for i in 0..qdcount {
        let (name, new_offset) = parse_name(data, offset)?;
        if i == 0 {
            domain = name;
        }
        offset = new_offset;
        offset += 4; // QTYPE + QCLASS
        if offset > data.len() {
            return None;
        }
    }

    // Parse answer section
    let mut addresses = Vec::new();
    for _ in 0..ancount {
        if offset >= data.len() {
            break;
        }

        let (_, new_offset) = parse_name(data, offset)?;
        offset = new_offset;

        if offset + 10 > data.len() {
            break;
        }

        let rtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let ttl = u32::from_be_bytes([
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
        ]);
        let rdlength = u16::from_be_bytes([data[offset + 8], data[offset + 9]]) as usize;
        offset += 10;

        if offset + rdlength > data.len() {
            break;
        }

        match rtype {
            1 if rdlength == 4 => {
                let ip = Ipv4Addr::new(
                    data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                );
                addresses.push((IpAddr::V4(ip), ttl));
            }
            28 if rdlength == 16 => {
                let mut addr_bytes = [0u8; 16];
                addr_bytes.copy_from_slice(&data[offset..offset + 16]);
                let ip = Ipv6Addr::from(addr_bytes);
                addresses.push((IpAddr::V6(ip), ttl));
            }
            _ => {}
        }

        offset += rdlength;
    }

    if addresses.is_empty() {
        return None;
    }

    Some(DnsResponse { domain, addresses })
}

fn parse_name(data: &[u8], mut offset: usize) -> Option<(String, usize)> {
    let mut name = String::new();
    let mut jumped = false;
    let mut return_offset = 0;
    let mut steps = 0;

    loop {
        if offset >= data.len() || steps > 128 {
            return None;
        }
        steps += 1;

        let len = data[offset] as usize;

        if len == 0 {
            if !jumped {
                return_offset = offset + 1;
            }
            break;
        }

        if len & 0xC0 == 0xC0 {
            if offset + 1 >= data.len() {
                return None;
            }
            if !jumped {
                return_offset = offset + 2;
            }
            let ptr = ((len & 0x3F) << 8) | (data[offset + 1] as usize);
            offset = ptr;
            jumped = true;
            continue;
        }

        offset += 1;
        if offset + len > data.len() {
            return None;
        }

        if !name.is_empty() {
            name.push('.');
        }
        name.push_str(std::str::from_utf8(&data[offset..offset + len]).ok()?);
        offset += len;
    }

    if !jumped {
        return_offset = offset + 1;
    }

    Some((name, return_offset))
}
