use uuid::Uuid;

pub(crate) type ServerId = Uuid;
pub(crate) type ChunkId = Uuid;
pub(crate) type Hostname = String;
pub(crate) type RackId = String;

pub(crate) struct Chunk {
    // TODO: add data for verifying ownership
    pub(crate) size: u64,
}
