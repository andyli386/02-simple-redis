use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};
use bytes::BytesMut;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

impl SimpleError {
    pub fn new(e: impl Into<String>) -> Self {
        SimpleError(e.into())
    }
}

impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_simple_frame_data(buf, Self::PREFIX)? {
            Some(end) => {
                let data = buf.split_to(end + 2);
                let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
                Ok(SimpleError::new(s.to_string()))
            }
            None => Err(RespError::NotComplete),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        match end {
            Some(end) => Ok(end + CRLF_LEN),
            None => Err(RespError::NotComplete),
        }
    }
}

impl From<&str> for SimpleError {
    fn from(value: &str) -> Self {
        SimpleError(value.to_string())
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{RespDecode, RespError, RespFrame, SimpleString};
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_error_encode() {
        let frame: RespFrame = SimpleError::new("Error message".to_string()).into();
        assert_eq!(frame.encode(), b"-Error message\r\n");
    }

    #[test]
    fn test_simple_error_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+hello\r");
        let ret = SimpleString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = SimpleString::decode(&mut buf);
        assert_eq!(frame.unwrap(), SimpleString::new("hello".to_string()));
        Ok(())
    }
}
