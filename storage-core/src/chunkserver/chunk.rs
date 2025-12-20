use uuid::Uuid;

pub(crate) type ChunkId = Uuid;

pub(crate) struct Chunk {
    pub(crate) id: ChunkId,
    pub(crate) size: u64,
}
