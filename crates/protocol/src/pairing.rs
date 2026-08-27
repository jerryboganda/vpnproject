//! Pairing QR Code Payload and Verification

use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::auth::{compute_hmac_tag, verify_hmac_tag};
use vpnbridge_core::error::{Error, Result};

/// Ephemeral QR Pairing payload exchanged between Android hotspot and Windows companion.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingPayload {
    pub ssid: String,
    pub gateway_ip: IpAddr,
    pub port: u16,
    pub token: String,
    pub created_at_secs: u64,
    pub ttl_secs: u64,
    pub fingerprint: String,
}

impl PairingPayload {
    /// Create a new pairing payload with a default 5-minute TTL.
    pub fn new(ssid: String, gateway_ip: IpAddr, port: u16, token: String, secret_key: &[u8]) -> Self {
        let created_at_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ttl_secs = 300; // 5 minutes

        let raw_data = format!("{}:{}:{}:{}:{}:{}", ssid, gateway_ip, port, token, created_at_secs, ttl_secs);
        let fingerprint = compute_hmac_tag(secret_key, raw_data.as_bytes());

        Self {
            ssid,
            gateway_ip,
            port,
            token,
            created_at_secs,
            ttl_secs,
            fingerprint,
        }
    }

    /// Validate the cryptographic signature and expiration of the pairing payload.
    pub fn validate(&self, secret_key: &[u8]) -> Result<()> {
        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now_secs > self.created_at_secs + self.ttl_secs {
            return Err(Error::AuthenticationFailed("Pairing QR payload has expired".to_string()));
        }

        let raw_data = format!("{}:{}:{}:{}:{}:{}", self.ssid, self.gateway_ip, self.port, self.token, self.created_at_secs, self.ttl_secs);
        if !verify_hmac_tag(secret_key, raw_data.as_bytes(), &self.fingerprint) {
            return Err(Error::AuthenticationFailed("Invalid pairing payload fingerprint".to_string()));
        }

        Ok(())
    }

    /// Encode payload into a compact URI for QR code generation:
    /// `vpnbridge://pair?ssid=...&gw=...&port=...&token=...&ts=...&ttl=...&fp=...`
    pub fn to_uri(&self) -> String {
        format!(
            "vpnbridge://pair?ssid={}&gw={}&port={}&token={}&ts={}&ttl={}&fp={}",
            url_encode(&self.ssid),
            self.gateway_ip,
            self.port,
            url_encode(&self.token),
            self.created_at_secs,
            self.ttl_secs,
            url_encode(&self.fingerprint),
        )
    }

    /// Parse a `vpnbridge://pair?...` URI into a `PairingPayload`.
    pub fn from_uri(uri: &str) -> Result<Self> {
        let stripped = uri.strip_prefix("vpnbridge://pair?").ok_or_else(|| {
            Error::ProtocolError("URI must start with vpnbridge://pair?".to_string())
        })?;

        let mut ssid = None;
        let mut gateway_ip = None;
        let mut port = None;
        let mut token = None;
        let mut created_at_secs = None;
        let mut ttl_secs = None;
        let mut fingerprint = None;

        for pair in stripped.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");

            match key {
                "ssid" => ssid = url_decode(val),
                "gw" => gateway_ip = val.parse().ok(),
                "port" => port = val.parse().ok(),
                "token" => token = url_decode(val),
                "ts" => created_at_secs = val.parse().ok(),
                "ttl" => ttl_secs = val.parse().ok(),
                "fp" => fingerprint = url_decode(val),
                _ => {}
            }
        }

        Ok(Self {
            ssid: ssid.ok_or_else(|| Error::ProtocolError("Missing ssid in URI".to_string()))?,
            gateway_ip: gateway_ip.ok_or_else(|| Error::ProtocolError("Missing gw in URI".to_string()))?,
            port: port.ok_or_else(|| Error::ProtocolError("Missing port in URI".to_string()))?,
            token: token.ok_or_else(|| Error::ProtocolError("Missing token in URI".to_string()))?,
            created_at_secs: created_at_secs.ok_or_else(|| Error::ProtocolError("Missing ts in URI".to_string()))?,
            ttl_secs: ttl_secs.ok_or_else(|| Error::ProtocolError("Missing ttl in URI".to_string()))?,
            fingerprint: fingerprint.ok_or_else(|| Error::ProtocolError("Missing fp in URI".to_string()))?,
        })
    }
}

fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    out
}

fn url_decode(input: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next()?;
            let h2 = chars.next()?;
            let hex_slice = [h1, h2];
            let hex_str = std::str::from_utf8(&hex_slice).ok()?;
            let val = u8::from_str_radix(hex_str, 16).ok()?;
            bytes.push(val);
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_payload_uri_roundtrip() {
        let secret = b"super-secure-key";
        let payload = PairingPayload::new(
            "VPNBridge-Hotspot-5G".to_string(),
            "192.168.43.1".parse().unwrap(),
            10808,
            "session-token-xyz".to_string(),
            secret,
        );

        let uri = payload.to_uri();
        assert!(uri.starts_with("vpnbridge://pair?"));

        let parsed = PairingPayload::from_uri(&uri).expect("Failed to parse URI");
        assert_eq!(payload, parsed);
        assert!(parsed.validate(secret).is_ok());
    }

    #[test]
    fn test_pairing_payload_invalid_secret() {
        let secret = b"super-secure-key";
        let bad_secret = b"wrong-key";
        let payload = PairingPayload::new(
            "VPNBridge-Hotspot".to_string(),
            "192.168.43.1".parse().unwrap(),
            10808,
            "token-123".to_string(),
            secret,
        );

        assert!(payload.validate(bad_secret).is_err());
    }
}
