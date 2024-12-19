use super::{parse_length, RespDecode, RespEncode, RespError, CRLF_LEN};
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BulkString(pub(crate) Option<Vec<u8>>);

impl BulkString {
    pub fn new(b: impl Into<Vec<u8>>) -> Self {
        BulkString(Some(b.into()))
    }

    pub fn null() -> Self {
        BulkString(None)
    }
}

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        match self.0 {
            Some(data) => {
                let mut buf = Vec::with_capacity(data.len() + 16);
                buf.extend_from_slice(&format!("${}\r\n", data.len()).into_bytes());
                buf.extend_from_slice(&data);
                buf.extend_from_slice(b"\r\n");
                buf
            }
            None => b"$-1\r\n".to_vec(),
        }
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.starts_with(b"$-1\r\n") {
            return Ok(BulkString::null());
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        if buf.starts_with(b"$-1\r\n") {
            return Ok(5);
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

impl From<&str> for BulkString {
    fn from(value: &str) -> Self {
        BulkString(Some(value.as_bytes().to_vec()))
    }
}

impl From<&[u8]> for BulkString {
    fn from(value: &[u8]) -> Self {
        BulkString(Some(value.to_vec()))
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(value: &[u8; N]) -> Self {
        BulkString(Some(value.to_vec()))
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        match &self.0 {
            Some(b) => b,
            None => &[],
        }
    }
}

impl Deref for BulkString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            Some(data) => data.as_slice(),
            None => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespDecode, RespEncode, RespError, RespFrame};
    use bytes::BytesMut;

    #[test]
    fn test_bulkstring_encode() {
        let frame: RespFrame = BulkString::new(b"hello".to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        Ok(())
    }

    #[test]
    fn test_null_bulkstring_encode() {
        let frame: RespFrame = BulkString::null().into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    #[test]
    fn test_null_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::null());

        Ok(())
    }
}
