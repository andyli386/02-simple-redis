use super::{extract_simple_frame_data, RespDecode, RespEncode, RespError, CRLF_LEN};
use bytes::BytesMut;

impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_simple_frame_data(buf, Self::PREFIX)? {
            Some(end) => {
                let data = buf.split_to(end + 2);
                let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
                Ok(s.parse()?)
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

#[cfg(test)]
mod tests {
    use crate::{RespDecode, RespEncode, RespError, RespFrame};
    use bytes::BytesMut;

    #[test]
    fn test_double_encode() {
        let frame: RespFrame = 123.456.into();
        assert_eq!(frame.encode(), b",+123.456\r\n");
        let frame: RespFrame = (-123.456).into();
        assert_eq!(frame.encode(), b",-123.456\r\n");
        let frame: RespFrame = 1.23456e+8.into();
        assert_eq!(frame.encode(), b",+1.23456e8\r\n");
        let frame: RespFrame = (-1.23456e-9).into();
        assert_eq!(frame.encode(), b",-1.23456e-9\r\n");
    }

    #[test]
    fn test_double_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",1234.45\r\n");
        let ret = f64::decode(&mut buf);
        assert_eq!(ret.unwrap(), 1234.45);

        buf.extend_from_slice(b",1.23456e-9\r\n");
        let ret = f64::decode(&mut buf);
        assert_eq!(ret.unwrap(), 1.23456e-9);

        buf.extend_from_slice(b"1.23456e-9\r\n");
        let ret = f64::decode(&mut buf);
        assert_eq!(
            ret,
            Err(RespError::InvalidFrame(format!(
                "expect: SimpleString(+), got: {:?}",
                buf.to_vec()
            )))
        );
        Ok(())
    }
}
