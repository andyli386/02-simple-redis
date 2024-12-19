use crate::{backend::Backend, BulkString, RespArray, RespFrame};

use super::{extract_args, validate_command, CommandError, CommandExecutor, Echo};

impl CommandExecutor for Echo {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.echo(&self.value)
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(BulkString(Some(v)))) => Ok(Echo {
                value: String::from_utf8(v)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}
