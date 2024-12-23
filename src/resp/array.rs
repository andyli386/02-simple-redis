use super::{
    calc_total_length, parse_length, RespDecode, RespEncode, RespError, RespFrame, BUF_CAP,
    CRLF_LEN,
};
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, Hash, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Option<Vec<RespFrame>>);

impl RespArray {
    pub fn new(a: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(Some(a.into()))
    }

    pub fn null() -> Self {
        RespArray(None)
    }
}

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        match self.0 {
            Some(arr) => {
                let mut buf = Vec::with_capacity(BUF_CAP);
                buf.extend_from_slice(&format!("*{}\r\n", arr.len()).into_bytes());
                for frame in arr {
                    buf.extend_from_slice(&frame.encode());
                }
                buf
            }
            None => b"*-1\r\n".to_vec(),
        }
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        if buf.starts_with(b"*-1\r\n") {
            return Ok(RespArray::null());
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }
        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        if buf.starts_with(b"$-1\r\n") {
            return Ok(4);
        }

        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Deref for RespArray {
    type Target = [RespFrame];

    fn deref(&self) -> &Self::Target {
        // &self.0
        match &self.0 {
            Some(data) => data.as_slice(),
            None => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, RespArray, RespDecode, RespEncode, RespError, RespFrame};
    use bytes::BytesMut;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }

    #[test]
    fn test_resp_null_array_encode() {
        let frame: RespFrame = RespArray::null().into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_null_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::null());

        Ok(())
    }
}
