use super::{extract_args, validate_command, CommandError, CommandExecutor, SAdd, SIsmember};
use crate::{backend::Backend, BulkString, RespArray, RespFrame, SimpleError};

impl CommandExecutor for SAdd {
    fn execute(self, backend: &Backend) -> RespFrame {
        match backend.sadd(self.key, self.value) {
            Some(true) => RespFrame::Integer(1),
            Some(false) => RespFrame::Error(SimpleError("sadd error!".into())),
            None => RespFrame::Integer(0),
        }
    }
}

impl CommandExecutor for SIsmember {
    fn execute(self, backend: &Backend) -> RespFrame {
        match backend.sismember(self.key, &self.value) {
            true => RespFrame::Integer(1),
            false => RespFrame::Integer(0),
        }
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(BulkString(Some(key)))), Some(value)) => Ok(SAdd {
                key: String::from_utf8(key)?,
                value,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for SIsmember {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(BulkString(Some(key)))), Some(value)) => Ok(SIsmember {
                key: String::from_utf8(key)?,
                value,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}
