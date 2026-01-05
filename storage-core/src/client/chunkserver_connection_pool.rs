use crate::types::ChunkserverId;
use moka::future::Cache;
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use std::sync::Arc;
use std::time::Duration;
use storage_core::common::types::ChunkserverLocation;

#[derive(Debug, Clone)]
pub(super) struct ChunkserverConnectionPool {
    endpoint: Arc<Endpoint>,
    chunkserver_connections: Cache<ChunkserverId, Arc<Connection>>,
}

impl ChunkserverConnectionPool {
    pub(super) fn new(endpoint: Arc<Endpoint>, max_capacity: u64, time_to_live: Duration) -> Self {
        ChunkserverConnectionPool {
            endpoint,
            chunkserver_connections: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(time_to_live)
                .build(),
        }
    }

    pub(super) async fn get_chunkserver_stream(
        &self,
        chunkserver_location: ChunkserverLocation,
    ) -> anyhow::Result<(SendStream, RecvStream)> {
        // Opens streams on either existing connection or on newly created connection
        // (lambda is run if the connection wasn't in Cache).
        self.chunkserver_connections
            .try_get_with::<_, anyhow::Error>(chunkserver_location.id, async {
                let conn = Arc::new(
                    self.endpoint
                        .connect(chunkserver_location.addr, &chunkserver_location.hostname)?
                        .await?,
                );
                self.chunkserver_connections
                    .insert(chunkserver_location.id, conn.clone())
                    .await;
                Ok(conn)
            })
            .await
            .map_err(|_| anyhow::anyhow!("Cache error"))?
            .open_bi()
            .await
            .map_err(|_| anyhow::anyhow!("Cache error"))
    }
}
