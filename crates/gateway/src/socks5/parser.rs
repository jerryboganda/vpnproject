//! Strict SOCKS5 & Username/Password Protocol Parser (RFC 1928 / RFC 1929)

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use vpnbridge_core::error::{Error, Result};

pub const SOCKS5_VERSION: u8 = 0x05;
pub const USER_PASS_AUTH_VERSION: u8 = 0x01;

pub const AUTH_METHOD_NONE: u8 = 0x00;
pub const AUTH_METHOD_USER_PASS: u8 = 0x02;
pub const AUTH_METHOD_NO_ACCEPTABLE: u8 = 0xFF;

pub const CMD_CONNECT: u8 = 0x01;
pub const CMD_BIND: u8 = 0x02;
pub const CMD_UDP_ASSOCIATE: u8 = 0x03;

pub const ATYP_IPV4: u8 = 0x01;
pub const ATYP_DOMAIN: u8 = 0x03;
pub const ATYP_IPV6: u8 = 0x04;

pub const REP_SUCCESS: u8 = 0x00;
pub const REP_GENERAL_FAILURE: u8 = 0x01;
pub const REP_CONN_NOT_ALLOWED: u8 = 0x02;
pub const REP_NETWORK_UNREACHABLE: u8 = 0x03;
pub const REP_HOST_UNREACHABLE: u8 = 0x04;
pub const REP_CONN_REFUSED: u8 = 0x05;
pub const REP_CMD_NOT_SUPPORTED: u8 = 0x07;
pub const REP_ATYP_NOT_SUPPORTED: u8 = 0x08;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Socks5Command {
    Connect,
    Bind,
    UdpAssociate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetAddress {
    Socket(SocketAddr),
    Domain(String, u16),
}

impl std::fmt::Display for TargetAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetAddress::Socket(addr) => write!(f, "{}", addr),
            TargetAddress::Domain(domain, port) => write!(f, "{}:{}", domain, port),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Socks5Request {
    pub command: Socks5Command,
    pub target: TargetAddress,
}

/// Perform initial SOCKS5 handshake and negotiation.
pub async fn read_client_greeting<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Vec<u8>> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read SOCKS5 greeting header: {e}"))
    })?;

    if header[0] != SOCKS5_VERSION {
        return Err(Error::Socks5Error(format!(
            "Invalid SOCKS version: expected {SOCKS5_VERSION}, got {}",
            header[0]
        )));
    }

    let nmethods = header[1] as usize;
    if nmethods == 0 {
        return Err(Error::Socks5Error("Client offered 0 authentication methods".to_string()));
    }

    let mut methods = vec![0u8; nmethods];
    stream.read_exact(&mut methods).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read SOCKS5 methods: {e}"))
    })?;

    Ok(methods)
}

/// Send selected authentication method response to client.
pub async fn write_server_method<S: AsyncWrite + Unpin>(stream: &mut S, method: u8) -> Result<()> {
    let response = [SOCKS5_VERSION, method];
    stream.write_all(&response).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to write SOCKS5 method response: {e}"))
    })?;
    stream.flush().await.map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

/// Read and parse RFC 1929 username/password auth request.
pub async fn read_user_pass_auth<S: AsyncRead + Unpin>(stream: &mut S) -> Result<(String, String)> {
    let mut ver_and_ulen = [0u8; 2];
    stream.read_exact(&mut ver_and_ulen).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read auth header: {e}"))
    })?;

    if ver_and_ulen[0] != USER_PASS_AUTH_VERSION {
        return Err(Error::Socks5Error(format!(
            "Invalid auth sub-negotiation version: {}",
            ver_and_ulen[0]
        )));
    }

    let ulen = ver_and_ulen[1] as usize;
    let mut uname_bytes = vec![0u8; ulen];
    stream.read_exact(&mut uname_bytes).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read username: {e}"))
    })?;

    let mut plen_buf = [0u8; 1];
    stream.read_exact(&mut plen_buf).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read password length: {e}"))
    })?;

    let plen = plen_buf[0] as usize;
    let mut pass_bytes = vec![0u8; plen];
    stream.read_exact(&mut pass_bytes).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read password: {e}"))
    })?;

    let username = String::from_utf8_lossy(&uname_bytes).into_owned();
    let password = String::from_utf8_lossy(&pass_bytes).into_owned();

    Ok((username, password))
}

/// Send RFC 1929 username/password auth status response.
pub async fn write_user_pass_auth_response<S: AsyncWrite + Unpin>(
    stream: &mut S,
    success: bool,
) -> Result<()> {
    let status = if success { 0x00 } else { 0x01 };
    let response = [USER_PASS_AUTH_VERSION, status];
    stream.write_all(&response).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to write auth status: {e}"))
    })?;
    stream.flush().await.map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

/// Read SOCKS5 client request (CONNECT, UDP ASSOCIATE, etc.)
pub async fn read_client_request<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Socks5Request> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to read SOCKS5 request header: {e}"))
    })?;

    if header[0] != SOCKS5_VERSION {
        return Err(Error::Socks5Error(format!(
            "Invalid SOCKS version in request: {}",
            header[0]
        )));
    }

    let command = match header[1] {
        CMD_CONNECT => Socks5Command::Connect,
        CMD_BIND => Socks5Command::Bind,
        CMD_UDP_ASSOCIATE => Socks5Command::UdpAssociate,
        cmd => return Err(Error::Socks5Error(format!("Unsupported SOCKS5 command: 0x{cmd:02x}"))),
    };

    let target = match header[3] {
        ATYP_IPV4 => {
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read IPv4 address: {e}"))
            })?;
            let mut port_buf = [0u8; 2];
            stream.read_exact(&mut port_buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read port: {e}"))
            })?;
            let port = u16::from_be_bytes(port_buf);
            let ip = Ipv4Addr::from(buf);
            TargetAddress::Socket(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        }
        ATYP_DOMAIN => {
            let mut len_buf = [0u8; 1];
            stream.read_exact(&mut len_buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read domain length: {e}"))
            })?;
            let domain_len = len_buf[0] as usize;
            if domain_len == 0 {
                return Err(Error::Socks5Error("Zero length domain name in SOCKS5 request".to_string()));
            }
            let mut domain_buf = vec![0u8; domain_len];
            stream.read_exact(&mut domain_buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read domain name: {e}"))
            })?;
            let mut port_buf = [0u8; 2];
            stream.read_exact(&mut port_buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read port: {e}"))
            })?;
            let port = u16::from_be_bytes(port_buf);
            let domain = String::from_utf8(domain_buf).map_err(|e| {
                Error::Socks5Error(format!("Invalid UTF-8 in domain name: {e}"))
            })?;
            TargetAddress::Domain(domain, port)
        }
        ATYP_IPV6 => {
            let mut buf = [0u8; 16];
            stream.read_exact(&mut buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read IPv6 address: {e}"))
            })?;
            let mut port_buf = [0u8; 2];
            stream.read_exact(&mut port_buf).await.map_err(|e| {
                Error::Socks5Error(format!("Failed to read port: {e}"))
            })?;
            let port = u16::from_be_bytes(port_buf);
            let ip = Ipv6Addr::from(buf);
            TargetAddress::Socket(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)))
        }
        atyp => {
            return Err(Error::Socks5Error(format!(
                "Unsupported SOCKS5 address type: 0x{atyp:02x}"
            )));
        }
    };

    Ok(Socks5Request { command, target })
}

/// Write SOCKS5 response packet to client.
pub async fn write_server_response<S: AsyncWrite + Unpin>(
    stream: &mut S,
    rep: u8,
    bound_addr: SocketAddr,
) -> Result<()> {
    let mut response = Vec::with_capacity(32);
    response.push(SOCKS5_VERSION);
    response.push(rep);
    response.push(0x00); // RSV

    match bound_addr {
        SocketAddr::V4(addr_v4) => {
            response.push(ATYP_IPV4);
            response.extend_from_slice(&addr_v4.ip().octets());
            response.extend_from_slice(&addr_v4.port().to_be_bytes());
        }
        SocketAddr::V6(addr_v6) => {
            response.push(ATYP_IPV6);
            response.extend_from_slice(&addr_v6.ip().octets());
            response.extend_from_slice(&addr_v6.port().to_be_bytes());
        }
    }

    stream.write_all(&response).await.map_err(|e| {
        Error::Socks5Error(format!("Failed to write SOCKS5 server response: {e}"))
    })?;
    stream.flush().await.map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

/// Parse SOCKS5 UDP encapsulation header from client datagram.
pub fn parse_udp_header(data: &[u8]) -> Result<(TargetAddress, usize)> {
    if data.len() < 4 {
        return Err(Error::Socks5Error("UDP packet too short for SOCKS5 header".to_string()));
    }

    if data[0] != 0x00 || data[1] != 0x00 {
        return Err(Error::Socks5Error("Invalid reserved bytes in UDP header".to_string()));
    }

    let frag = data[2];
    if frag != 0x00 {
        return Err(Error::Socks5Error("UDP fragmentation not supported".to_string()));
    }

    let atyp = data[3];
    let (target, offset) = match atyp {
        ATYP_IPV4 => {
            if data.len() < 10 {
                return Err(Error::Socks5Error("UDP IPv4 packet too short".to_string()));
            }
            let mut ip_bytes = [0u8; 4];
            ip_bytes.copy_from_slice(&data[4..8]);
            let port = u16::from_be_bytes([data[8], data[9]]);
            let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from(ip_bytes), port));
            (TargetAddress::Socket(addr), 10)
        }
        ATYP_DOMAIN => {
            if data.len() < 5 {
                return Err(Error::Socks5Error("UDP domain header too short".to_string()));
            }
            let domain_len = data[4] as usize;
            if data.len() < 5 + domain_len + 2 {
                return Err(Error::Socks5Error("UDP packet truncated domain".to_string()));
            }
            let domain_bytes = &data[5..5 + domain_len];
            let domain = String::from_utf8_lossy(domain_bytes).into_owned();
            let port = u16::from_be_bytes([
                data[5 + domain_len],
                data[5 + domain_len + 1],
            ]);
            (TargetAddress::Domain(domain, port), 5 + domain_len + 2)
        }
        ATYP_IPV6 => {
            if data.len() < 22 {
                return Err(Error::Socks5Error("UDP IPv6 packet too short".to_string()));
            }
            let mut ip_bytes = [0u8; 16];
            ip_bytes.copy_from_slice(&data[4..20]);
            let port = u16::from_be_bytes([data[20], data[21]]);
            let addr = SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from(ip_bytes), port, 0, 0));
            (TargetAddress::Socket(addr), 22)
        }
        _ => return Err(Error::Socks5Error(format!("Unsupported UDP address type: 0x{atyp:02x}"))),
    };

    Ok((target, offset))
}

/// Construct SOCKS5 UDP encapsulation header for datagram sent back to client.
pub fn build_udp_header(target: &TargetAddress, output: &mut Vec<u8>) {
    output.push(0x00); // RSV
    output.push(0x00); // RSV
    output.push(0x00); // FRAG

    match target {
        TargetAddress::Socket(SocketAddr::V4(addr_v4)) => {
            output.push(ATYP_IPV4);
            output.extend_from_slice(&addr_v4.ip().octets());
            output.extend_from_slice(&addr_v4.port().to_be_bytes());
        }
        TargetAddress::Socket(SocketAddr::V6(addr_v6)) => {
            output.push(ATYP_IPV6);
            output.extend_from_slice(&addr_v6.ip().octets());
            output.extend_from_slice(&addr_v6.port().to_be_bytes());
        }
        TargetAddress::Domain(domain, port) => {
            output.push(ATYP_DOMAIN);
            output.push(domain.len() as u8);
            output.extend_from_slice(domain.as_bytes());
            output.extend_from_slice(&port.to_be_bytes());
        }
    }
}
