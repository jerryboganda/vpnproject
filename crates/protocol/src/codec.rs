//! Length-Delimited Protocol Codec for Stream I/O

use crate::messages::ControlMessage;
use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use vpnbridge_core::error::{Error, Result};

pub const MAX_CONTROL_FRAME_SIZE: usize = 65536; // 64KB max control message limit

/// Length-delimited binary JSON frame codec.
#[derive(Default)]
pub struct ProtocolCodec;

impl Decoder for ProtocolCodec {
    type Item = ControlMessage;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        if length > MAX_CONTROL_FRAME_SIZE {
            return Err(Error::ProtocolError(format!(
                "Control frame length {length} exceeds maximum limit {MAX_CONTROL_FRAME_SIZE}"
            )));
        }

        if src.len() < 4 + length {
            // Wait for complete frame
            src.reserve(4 + length - src.len());
            return Ok(None);
        }

        src.advance(4);
        let payload = src.split_to(length);

        let message: ControlMessage = serde_json::from_slice(&payload)
            .map_err(|e| Error::ProtocolError(format!("Failed to deserialize control message: {e}")))?;

        Ok(Some(message))
    }
}

impl Encoder<ControlMessage> for ProtocolCodec {
    type Error = Error;

    fn encode(&mut self, item: ControlMessage, dst: &mut BytesMut) -> Result<()> {
        let payload = serde_json::to_vec(&item)
            .map_err(|e| Error::ProtocolError(format!("Failed to serialize control message: {e}")))?;

        if payload.len() > MAX_CONTROL_FRAME_SIZE {
            return Err(Error::CapacityExceeded(format!(
                "Control frame payload {} exceeds max frame size {}",
                payload.len(),
                MAX_CONTROL_FRAME_SIZE
            )));
        }

        let length = payload.len() as u32;
        dst.reserve(4 + payload.len());
        dst.put_u32(length);
        dst.put_slice(&payload);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{HandshakeRequest, CURRENT_PROTOCOL_VERSION};

    #[test]
    fn test_codec_roundtrip() {
        let mut codec = ProtocolCodec;
        let mut buffer = BytesMut::new();

        let req = ControlMessage::HandshakeReq(HandshakeRequest {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            client_id: "client-win-1".to_string(),
            device_name: "Windows Laptop".to_string(),
            nonce: "rand-nonce".to_string(),
        });

        codec.encode(req.clone(), &mut buffer).expect("Encode should succeed");
        let decoded = codec.decode(&mut buffer).expect("Decode should succeed").expect("Must have frame");
        assert_eq!(decoded, req);
    }
}
