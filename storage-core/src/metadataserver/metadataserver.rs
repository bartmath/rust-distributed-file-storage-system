use quinn::{Connection, Endpoint};
use std::net::SocketAddr;
use uuid::Uuid;

type Hostname = String;
type RackId = String;

pub(crate) struct ChunkserverMetadata {
    pub hostname: Hostname,
    rack_id: RackId,
    pub address: SocketAddr,
    connection: Connection,
}

struct MetadataServer {
    // The id of the metadataserver is assigned
    id: Option<Uuid>,
    chunk_servers: Vec<ChunkserverMetadata>,

    client_endpoint: Endpoint,
    chunkservers_endpoint: Endpoint,
}

impl MetadataServer {}
