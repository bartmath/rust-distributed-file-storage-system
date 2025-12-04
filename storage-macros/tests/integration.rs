/* use tokio::io::{duplex, DuplexStream};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestChunkPayload {
    pub id: u32,
    #[serde(skip)]
    pub data: Vec<u8>,
}

// Constructor function for the macro
async fn add_chunk_data(payload: TestChunkPayload, chunk_data: Vec<u8>) -> Result<TestChunkPayload> {
    Ok(TestChunkPayload {
        id: payload.id,
        data: chunk_data,
    })
}

// Use macro to implement MessagePayload
impl_chunk_payload!(TestChunkPayload, add_chunk_data);

#[tokio::test]
async fn test_send_recv_chunk_payload() {
    // Create a duplex in-memory stream pair
    let (mut send, mut recv): (DuplexStream, DuplexStream) = duplex(1024);

    // Create test object
    let original = TestChunkPayload {
        id: 42,
        data: vec![1, 2, 3, 4, 5],
    };

    // Spawn sending task
    let send_task = tokio::spawn(async move {
        original.send_payload(&mut send).await.unwrap();
    });

    // Receive payload
    let received = TestChunkPayload::recv_payload(&mut recv).await?;

    // Wait for sender to finish
    send_task.await.expect("Couldn't send payload");

    // Assert fields equality
    assert_eq!(received.id, 42);
    assert_eq!(received.data, vec![1, 2, 3, 4, 5]);
} */
