use crate::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ByteCountSnapshotEntry {
    pub name: Option<Arc<str>>,
    pub key: MessageKey,
    pub lines: u64,
    pub bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ByteCountSnapshot {
    snapshot: Box<[ByteCountSnapshotEntry]>,
}

impl ByteCountSnapshot {
    pub fn empty() -> Self {
        Self {
            snapshot: Box::new([]),
        }
    }

    pub fn new(mut snapshot: Box<[ByteCountSnapshotEntry]>) -> Self {
        // To ensure there's a defined order for a much easier time testing.
        snapshot.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        Self { snapshot }
    }

    pub fn is_empty(&self) -> bool {
        self.snapshot.is_empty()
    }

    pub fn into_inner(self) -> Box<[ByteCountSnapshotEntry]> {
        self.snapshot
    }

    pub fn each_while(&self, receiver: impl FnMut(&ByteCountSnapshotEntry) -> bool) -> bool {
        self.snapshot.iter().all(receiver)
    }

    #[cfg(test)]
    pub fn build(data: impl IntoIterator<Item = ByteCountSnapshotEntry>) -> Self {
        Self::new(Box::from_iter(data))
    }
}
