use super::{CommandExecutor, HGetAll, HMGet, HSet, RESP_OK};
use crate::{
    backend::Backend,
    cmd::{extract_args, validate_command, CommandError, HGet},
    BulkString, RespArray, RespFrame, RespNull,
};

impl CommandExecutor for HGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        match backend.hget(&self.key, &self.field) {
            Some(value) => value,
            None => RespFrame::Null(RespNull),
        }
    }
}

impl CommandExecutor for HGetAll {
    fn execute(self, backend: &Backend) -> RespFrame {
        let hmap = backend.hmap.get(&self.key);
        match hmap {
            Some(hmap) => {
                let mut ret = Vec::with_capacity(hmap.len() * 2);
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    ret.push(BulkString::new(key).into());
                    ret.push(v.value().clone())
                }
                RespArray::new(ret).into()
            }
            None => RespArray::new([]).into(),
        }
    }
}

impl CommandExecutor for HMGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        let mut ret = Vec::with_capacity(self.fields.len());
        for field in self.fields {
            let hmap = backend.hget(&self.key, &field);
            match hmap {
                Some(value) => ret.push(value),
                None => ret.push(RespFrame::Null(RespNull)),
            }
        }
        RespArray::new(ret).into()
    }
}

impl CommandExecutor for HSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.hset(self.key, self.field, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for HGetAll {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HGetAll {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let capacity = value.len() - 1;
        validate_command(&value, &["hmget"], capacity)?;
        let mut args = extract_args(value, 1)?.into_iter();

        let mut fields = Vec::with_capacity(capacity);

        let key = if let Some(RespFrame::BulkString(k)) = args.next() {
            String::from_utf8(k.0)?
        } else {
            return Err(CommandError::InvalidArgument("Invalid key".to_string()));
        };

        for frame in args {
            match frame {
                RespFrame::BulkString(field) => fields.push(String::from_utf8(field.0)?),
                _ => return Err(CommandError::InvalidArgument("Invalid field".to_string())),
            };
        }

        Ok(HMGet { key, fields })
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], 3)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HSet {
                    key: String::from_utf8(key.0)?,
                    field: String::from_utf8(field.0)?,
                    value,
                })
            }
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use dashmap::DashMap;

    use crate::{
        backend::Backend,
        cmd::{CommandExecutor, HGet, HGetAll, HSet},
        BulkString, RespArray, RespDecode, RespFrame,
    };

    #[test]
    fn test_hget_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HGet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");

        Ok(())
    }

    #[test]
    fn test_hgetall_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HSet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));
        Ok(())
    }

    #[test]
    fn test_hset_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HGetAll = frame.try_into()?;
        assert_eq!(result.key, "map");
        Ok(())
    }

    #[test]
    fn test_hgetall_existing_key() {
        let backend = Backend::new();
        let hmap = DashMap::new();
        hmap.insert("field1".to_string(), BulkString::new("value1").into());
        hmap.insert("field2".to_string(), BulkString::new("value2").into());
        backend.hmap.insert("myhash".to_string(), hmap);

        let command = HGetAll {
            key: "myhash".to_string(),
        };
        let result = command.execute(&backend);

        assert!(matches!(result, RespFrame::Array(_)));
        if let RespFrame::Array(array) = result {
            // 将结果分组为键值对并排序
            let mut pairs: Vec<_> = array
                .chunks(2)
                .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
                .collect();
            pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // 预期的键值对，按相同顺序排序
            let mut expected = vec![
                (
                    BulkString::new("field1").into(),
                    BulkString::new("value1").into(),
                ),
                (
                    BulkString::new("field2").into(),
                    BulkString::new("value2").into(),
                ),
            ];
            expected.sort_by(|a: &(RespFrame, RespFrame), b| a.0.partial_cmp(&b.0).unwrap());

            assert_eq!(pairs, expected);
        }
    }

    #[test]
    fn test_hgetall_nonexistent_key() {
        let backend = Backend::new();
        let command = HGetAll {
            key: "nonexistent".to_string(),
        };
        let result = command.execute(&backend);

        assert!(matches!(result, RespFrame::Array(array) if array.is_empty()));
    }
}
