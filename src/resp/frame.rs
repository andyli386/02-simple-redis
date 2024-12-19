use super::{
    BulkString, RespArray, RespDecode, RespError, RespMap, RespNull, RespNullArray, RespSet,
    SimpleError, SimpleString,
};
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use std::hash::Hash;

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

impl Eq for RespFrame {}
impl Hash for RespFrame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            RespFrame::SimpleString(s) => s.hash(state),
            RespFrame::Error(e) => e.hash(state),
            RespFrame::Integer(i) => i.hash(state),
            RespFrame::BulkString(bs) => bs.hash(state),
            RespFrame::Array(arr) => arr.hash(state),
            RespFrame::NullArray(_) => {}
            RespFrame::Null(_) => {}
            RespFrame::Boolean(b) => b.hash(state),
            RespFrame::Double(f) => f.to_bits().hash(state),
            RespFrame::Map(m) => m.hash(state),
            RespFrame::Set(s) => s.hash(state),
        }
    }
}

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
            Some(b'$') => match BulkString::decode(buf) {
                Ok(frame) => Ok(frame.into()),
                Err(e) => Err(e),
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
            None => Err(RespError::NotComplete),
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

impl From<&str> for RespFrame {
    fn from(value: &str) -> Self {
        SimpleString(value.to_string()).into()
    }
}

impl From<&[u8]> for RespFrame {
    fn from(value: &[u8]) -> Self {
        BulkString(Some(value.to_vec())).into()
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(value: &[u8; N]) -> Self {
        BulkString(Some(value.to_vec())).into()
    }
}
