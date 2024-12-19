use std::{fmt, ops::Deref, sync::Arc};

use dashmap::{DashMap, DashSet};

use crate::RespFrame;

#[derive(Debug, Clone)]
pub struct Backend(pub(crate) Arc<BackendInner>);

pub struct BackendInner {
    pub map: DashMap<String, RespFrame>,
    pub hmap: DashMap<String, DashMap<String, RespFrame>>,
    pub set: DashSet<RespFrame>,
}

impl fmt::Debug for BackendInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BackendInner")
            .field("map", &self.map)
            .field("hmap", &self.hmap)
            .field("set", &"<DashSet<RespFrame>>") // 将 DashSet 转为 Vec 输出
            .finish()
    }
}

impl Deref for Backend {
    type Target = BackendInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            hmap: DashMap::new(),
            set: DashSet::new(),
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        // let hmap = self.hmap.entry(key).or_insert_with(DashMap::new);
        let hmap = self.hmap.entry(key).or_default();
        hmap.insert(field, value);
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }

    pub fn echo(&self, value: &str) -> RespFrame {
        RespFrame::BulkString(value.into())
    }
}
