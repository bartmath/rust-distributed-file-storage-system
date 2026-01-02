use anyhow::Result;
use async_trait::async_trait;
use quinn::{Connecting, Endpoint, RecvStream, SendStream};

#[async_trait]
pub trait QuicServer: Send + Sync + Clone + 'static {
    fn listening_endpoint(&self) -> &Endpoint;

    async fn setup(&self) -> Result<()>;

    async fn run(&self) -> Result<()> {
        self.setup().await?;

        let endpoint = self.listening_endpoint();
        loop {
            if let Some(incoming) = endpoint.accept().await {
                if let Ok(connecting) = incoming.accept() {
                    let server_clone = self.clone();
                    tokio::spawn(async move {
                        server_clone.handle_connection_handshake(connecting).await
                    });
                }
            }
        }
    }

    async fn handle_connection_handshake(&self, connecting: Connecting) {
        match connecting.await {
            Ok(conn) => {
                if let Err(e) = self.handle_connection_loop(conn).await {
                    eprintln!("Connection loop error: {:?}", e);
                }
            }
            Err(e) => eprintln!("Handshake failed: {:?}", e),
        }
    }

    async fn handle_connection_loop(&self, conn: quinn::Connection) -> Result<()> {
        loop {
            let stream = match conn.accept_bi().await {
                Ok(s) => s,
                Err(quinn::ConnectionError::ApplicationClosed { .. }) => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            let (send, recv) = stream;
            self.handle_request(send, recv).await?;
        }
    }

    async fn handle_request(&self, send: SendStream, recv: RecvStream) -> Result<()>;
}
