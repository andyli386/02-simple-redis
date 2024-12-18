use super::{extract_fixed_data, RespDecode, RespEncode, RespError};
use bytes::BytesMut;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "_\r\n", "Null") {
            Ok(_) => Ok(RespNull),
            Err(e) => Err(e),
        }
    }

    fn expect_length(_: &[u8]) -> Result<usize, RespError> {
        Ok(3)
    }
}

#[cfg(test)]
mod test {
    use crate::{RespDecode, RespEncode, RespFrame, RespNull};
    use bytes::BytesMut;

    #[test]
    fn test_resp_null_encode() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n");
    }

    #[test]
    fn test_null_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"_\r\n");

        let frame = RespNull::decode(&mut buf)?;
        assert_eq!(frame, RespNull);

        Ok(())
    }
}
