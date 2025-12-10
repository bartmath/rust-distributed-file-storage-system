use moka::future::Cache;
use quinn::{Connection, Endpoint};
use std::net::SocketAddr;

type ServerLocation = SocketAddr;

type ServerConnections = Cache<ServerLocation, Connection>;
struct ClientState {
    endpoint: Endpoint,
    metadata_server_location: ServerLocation,
    metadata_server_connection: Option<Connection>,
    chunkserver_connections: ServerConnections,
}
