use super::CommandExecutor;
use crate::{
    backend::Backend,
    cmd::{extract_args, validate_command, CommandError, Get, Set},
    RespArray, RespFrame, RespNull, SimpleString,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

impl CommandExecutor for Get {
    fn execute(self, backend: &Backend) -> RespFrame {
        match backend.get(&self.key) {
            Some(value) => value,
            // None => RESP_OK.clone(),
            None => RespFrame::Null(RespNull),
        }
    }
}

impl CommandExecutor for Set {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.set(self.key, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Get {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(Set {
                key: String::from_utf8(key.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    use crate::{
        cmd::{Get, Set},
        RespArray, RespDecode, RespFrame,
    };

    #[test]
    fn test_get_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        let result: Get = frame.try_into()?;
        assert_eq!(result.key, "hello");

        Ok(())
    }

    #[test]
    fn test_set_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: Set = frame.try_into()?;
        assert_eq!(result.key, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::Backend,
        cmd::{map::RESP_OK, CommandExecutor, Get, Set},
        RespFrame,
    };

    #[test]
    fn test_set_get_cmd() -> anyhow::Result<()> {
        let backend = Backend::new();
        let cmd = Set {
            key: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = Get {
            key: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));
        Ok(())
    }
}