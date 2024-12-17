pub mod decode;
pub mod encode;

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, BytesMut};
use enum_dispatch::enum_dispatch;
use thiserror::Error;

#[enum_dispatch]
pub trait RespEncode {
    fn encode(self) -> Vec<u8>;
}

pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError>;
    fn expect_length(buf: &[u8]) -> Result<usize, RespError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RespError {
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(String),
    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(String),
    #[error("Frame is not complete")]
    NotComplete,

    #[error("Parse error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Parse error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    NullBulkString(RespNullBulkString),
    NullArray(RespNullArray),
    Null(RespNull),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct SimpleError(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullBulkString;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNullArray;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct RespNull;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

const BUF_CAP: usize = 4096;
const CRLF: &[u8] = b"\r\n";
const CRLF_LEN: usize = CRLF.len();

// find nth CRLF in the buffer
fn find_crlf(buf: &[u8], nth: usize) -> Option<usize> {
    let mut count = 0;
    for i in 1..buf.len() - 1 {
        if buf[i] == b'\r' && buf[i + 1] == b'\n' {
            count += 1;
            if count == nth {
                return Some(i);
            }
        }
    }
    None
}

fn parse_length(buf: &[u8], prefix: &str) -> Result<(usize, usize), RespError> {
    let end = extract_simple_frame_data(buf, prefix)?;
    if let Some(end) = end {
        let s = String::from_utf8_lossy(&buf[prefix.len()..end]);
        Ok((end, s.parse()?))
    } else {
        Err(RespError::NotComplete)
    }
}

fn calc_total_length(buf: &[u8], end: usize, len: usize, prefix: &str) -> Result<usize, RespError> {
    let mut total = end + CRLF_LEN;
    let mut data = &buf[total..];
    match prefix {
        "*" | "~" => {
            for _ in 0..len {
                let elem_len = RespFrame::expect_length(data)?;
                data = &data[elem_len..];
                total += elem_len;
            }
            Ok(total)
        }
        "%" => {
            for _ in 0..len {
                let key_len = SimpleString::expect_length(data)?;
                data = &data[key_len..];
                total += key_len;

                let value_len = RespFrame::expect_length(data)?;
                data = &data[value_len..];
                total += value_len;
            }
            Ok(total)
        }
        _ => Ok(len + CRLF_LEN),
    }
}

fn extract_simple_frame_data(buf: &[u8], prefix: &str) -> Result<Option<usize>, RespError> {
    if buf.len() < 3 {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(prefix.as_bytes()) {
        return Err(RespError::InvalidFrame(format!(
            "expect: SimpleString(+), got: {:?}",
            buf
        )));
    }

    let end = find_crlf(buf, 1).ok_or(RespError::NotComplete)?;

    Ok(Some(end))
}

fn extract_fixed_data(
    buf: &mut BytesMut,
    expect: &str,
    expect_type: &str,
) -> Result<(), RespError> {
    if buf.len() < expect.len() {
        return Err(RespError::NotComplete);
    }

    if !buf.starts_with(expect.as_bytes()) {
        return Err(RespError::InvalidFrameType(format!(
            "expect: {}, got: {:?}",
            expect_type, buf
        )));
    }

    buf.advance(expect.len());

    Ok(())
}

impl From<&str> for RespFrame {
    fn from(value: &str) -> Self {
        // RespFrame::SimpleString(SimpleString::new(value))
        // SimpleString::new(value).into()
        SimpleString(value.to_string()).into()
    }
}

impl From<&str> for SimpleString {
    fn from(value: &str) -> Self {
        SimpleString(value.to_string())
    }
}

impl From<&str> for SimpleError {
    fn from(value: &str) -> Self {
        SimpleError(value.to_string())
    }
}

impl From<&str> for BulkString {
    fn from(value: &str) -> Self {
        BulkString(value.as_bytes().to_vec())
    }
}

impl From<&[u8]> for BulkString {
    fn from(value: &[u8]) -> Self {
        BulkString(value.to_vec())
    }
}

impl From<&[u8]> for RespFrame {
    fn from(value: &[u8]) -> Self {
        BulkString(value.to_vec()).into()
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(value: &[u8; N]) -> Self {
        BulkString(value.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(value: &[u8; N]) -> Self {
        BulkString(value.to_vec()).into()
    }
}

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SimpleError {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl SimpleError {
    pub fn new(e: impl Into<String>) -> Self {
        SimpleError(e.into())
    }
}

impl BulkString {
    pub fn new(b: impl Into<Vec<u8>>) -> Self {
        BulkString(b.into())
    }
}

impl RespArray {
    pub fn new(a: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(a.into())
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

impl RespSet {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        let s = s.into();
        RespSet(s)
    }
}

impl AsRef<str> for SimpleString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
