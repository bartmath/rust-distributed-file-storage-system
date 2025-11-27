use quinn::Endpoint;
use uuid::Uuid;

struct ChunkServerInfo {
    server_id: Uuid,
    rack_id: Uuid,
    address: String,
}

struct Metadataserver {
    // The id of the metadataserver is assigned
    id: Option<Uuid>,
    chunk_servers: Vec<ChunkServerInfo>,

    clients_endpoint: Endpoint,
    chunkservers_endpoint: Endpoint,
}

impl Metadataserver {}