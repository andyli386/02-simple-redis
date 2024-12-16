use crate::RespDecode;
use bytes::{Buf, BytesMut};

use super::{
    calc_total_length, extract_fixed_data, extract_simple_frame_data, parse_length, BulkString,
    RespArray, RespError, RespFrame, RespMap, RespNull, RespNullArray, RespNullBulkString, RespSet,
    SimpleError, SimpleString, CRLF_LEN,
};

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'+') => {
                let frame = SimpleString::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'-') => {
                let frame = SimpleError::decode(buf)?;
                Ok(frame.into())
            }
            Some(b':') => {
                let frame = i64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'$') => match RespNullBulkString::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = BulkString::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(b'*') => match RespNullArray::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(RespError::NotComplete) => Err(RespError::NotComplete),
                Err(_) => {
                    let frame = RespArray::decode(buf)?;
                    Ok(frame.into())
                }
            },
            Some(b'_') => {
                let frame = RespNull::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'#') => {
                let frame = bool::decode(buf)?;
                Ok(frame.into())
            }
            Some(b',') => {
                let frame = f64::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'%') => {
                let frame = RespMap::decode(buf)?;
                Ok(frame.into())
            }
            Some(b'~') => {
                let frame = RespSet::decode(buf)?;
                Ok(frame.into())
            }
            _ => Err(RespError::InvalidFrameType(format!(
                "decode unknown frame type: {:?}",
                buf
            ))),
        }
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let mut iter = buf.iter().peekable();
        match iter.peek() {
            Some(b'*') => RespArray::expect_length(buf),
            Some(b'~') => RespSet::expect_length(buf),
            Some(b'%') => RespMap::expect_length(buf),
            Some(b'$') => BulkString::expect_length(buf),
            Some(b':') => i64::expect_length(buf),
            Some(b'+') => SimpleString::expect_length(buf),
            Some(b'-') => SimpleError::expect_length(buf),
            Some(b'#') => bool::expect_length(buf),
            Some(b',') => f64::expect_length(buf),
            Some(b'_') => RespNull::expect_length(buf),
            _ => Err(RespError::NotComplete),
        }
    }
}

impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let end = extract_simple_frame_data(buf, Self::PREFIX)?;
        match end {
            Some(end) => {
                let data = buf.split_to(end + 2);
                let s = String::from_utf8_lossy(&data[Self::PREFIX.len()..end]);
                Ok(SimpleString::new(s.to_string()))
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

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
        Ok(RespNullArray)
    }

    fn expect_length(_: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

impl RespDecode for RespNullBulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "$-1\r\n", "NullBulkString")?;
        Ok(RespNullBulkString)
    }

    fn expect_length(_: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
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

impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        match extract_fixed_data(buf, "#t\r\n", "Bool") {
            Ok(_) => Ok(true),
            Err(RespError::NotComplete) => Err(RespError::NotComplete),
            Err(_) => match extract_fixed_data(buf, "#f\r\n", "Bool") {
                Ok(_) => Ok(false),
                Err(e) => Err(e),
            },
        }
    }

    fn expect_length(_: &[u8]) -> Result<usize, RespError> {
        Ok(4)
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";

    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
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
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
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

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut frames = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            frames.insert(key.0, value);
        }
        Ok(frames)
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
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
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::new();
        for _ in 0..len {
            let value = RespFrame::decode(buf)?;
            frames.push(value);
        }
        Ok(RespSet::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        resp::{calc_total_length, parse_length},
        BulkString, RespArray, RespDecode, RespError, RespFrame, RespMap, RespNull, RespNullArray,
        RespNullBulkString, RespSet, SimpleString,
    };
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_simple_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"+OK\r\n");

        let frame = SimpleString::decode(&mut buf)?;
        assert_eq!(frame, SimpleString::new("OK".to_string()));

        buf.extend_from_slice(b"+hello\r");
        let ret = SimpleString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = SimpleString::decode(&mut buf);
        assert_eq!(frame.unwrap(), SimpleString::new("hello".to_string()));
        Ok(())
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

    #[test]
    fn test_double_decode1() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b",123.45\r\n");

        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 123.45);

        buf.extend_from_slice(b",+1.23456e-9\r\n");
        let frame = f64::decode(&mut buf)?;
        assert_eq!(frame, 1.23456e-9);

        Ok(())
    }

    #[test]
    fn test_integer_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b":+123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, 123);

        buf.extend_from_slice(b":-123\r\n");

        let frame = i64::decode(&mut buf)?;
        assert_eq!(frame, -123);

        Ok(())
    }

    #[test]
    fn test_null_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        let frame = RespNullArray::decode(&mut buf)?;
        assert_eq!(frame, RespNullArray);

        Ok(())
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
    fn test_boolean_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"#t\r\n");

        let frame = bool::decode(&mut buf)?;
        assert!(frame);

        buf.extend_from_slice(b"#f\r\n");

        let frame = bool::decode(&mut buf)?;
        println!("----- {:?}", frame);
        assert!(!frame);

        buf.extend_from_slice(b"#f\r");
        let ret = bool::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.put_u8(b'\n');
        let frame = bool::decode(&mut buf)?;
        assert!(!frame);

        Ok(())
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
    fn test_null_bulk_string_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$-1\r\n");

        let frame = RespNullBulkString::decode(&mut buf)?;
        assert_eq!(frame, RespNullBulkString);

        Ok(())
    }

    #[test]
    fn test_set_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n+hello\r\n,1234.45\r\n");
        let ret = RespSet::decode(&mut buf);
        assert_eq!(
            ret.unwrap(),
            RespSet::new(vec![
                RespFrame::SimpleString(SimpleString::new("hello".to_string())),
                RespFrame::Double(1234.45)
            ])
        );
        Ok(())
    }

    #[test]
    fn test_set_decode1() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespSet::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespSet::new(vec![
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into()
            ])
        );

        Ok(())
    }

    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");
        let frame = RespMap::decode(&mut buf)?;
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new(b"world".to_vec()).into(),
        );
        map.insert("foo".to_string(), b"bar".into());
        assert_eq!(frame, map);
        Ok(())
    }

    #[test]
    fn test_calc_array_length() -> anyhow::Result<()> {
        let buf = b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let total_len = calc_total_length(buf, end, len, "*")?;
        assert_eq!(total_len, buf.len());

        let buf = b"*2\r\n$3\r\nset\r\n";
        let (end, len) = parse_length(buf, "*")?;
        let ret = calc_total_length(buf, end, len, "*");
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);
        Ok(())
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
