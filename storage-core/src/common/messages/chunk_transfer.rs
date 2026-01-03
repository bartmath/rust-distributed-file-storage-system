use crate::common::config::TMP_STORAGE_ROOT;
use crate::common::types::ChunkId;
use quinn::{RecvStream, SendStream};
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter};

#[derive(Debug, Default)]
pub struct ChunkTransfer {
    pub offset: Option<u64>,
    pub data: PathBuf,
    should_delete: bool,
}

impl ChunkTransfer {
    pub fn new(offset: Option<u64>, data: PathBuf) -> Self {
        ChunkTransfer {
            offset,
            data,
            should_delete: false,
        }
    }

    pub fn commit(mut self) -> PathBuf {
        self.should_delete = false;
        self.data.clone()
    }

    pub(crate) async fn send_chunk(
        &self,
        chunk_size: u64,
        send: &mut SendStream,
    ) -> anyhow::Result<()> {
        let mut file = File::open(&self.data).await?;

        if let Some(offset) = self.offset {
            AsyncSeekExt::seek(&mut file, SeekFrom::Start(offset)).await?;
        }

        let mut file = file.take(chunk_size);

        let bytes_sent = tokio::io::copy(&mut file, send).await?;

        if bytes_sent < chunk_size {
            anyhow::bail!("Chunk read to few bytes");
        }

        Ok(())
    }

    pub(crate) async fn recv_chunk(
        &self,
        chunk_size: u64,
        file_writer: &mut BufWriter<File>,
        recv: &mut RecvStream,
    ) -> anyhow::Result<()> {
        let mut limited_recv = recv.take(chunk_size);
        tokio::io::copy(&mut limited_recv, file_writer).await?;
        file_writer.flush().await?;

        Ok(())
    }
}

impl Drop for ChunkTransfer {
    fn drop(&mut self) {
        if self.should_delete && self.data.exists() {
            let _ = std::fs::remove_file(&self.data);
        }
    }
}
