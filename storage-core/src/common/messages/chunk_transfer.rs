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
}

impl ChunkTransfer {
    pub(crate) async fn send_chunk(
        &self,
        chunk_size: u64,
        send: &mut SendStream,
    ) -> anyhow::Result<()> {
        let mut file = tokio::fs::File::open(&self.data).await?;

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
        chunk_id: ChunkId,
        chunk_size: u64,
        recv: &mut RecvStream,
    ) -> anyhow::Result<Self> {
        let data = TMP_STORAGE_ROOT
            .get()
            .expect("Temporary storage not initialized via config")
            .join(chunk_id.to_string());

        let file = File::create(&data).await?;
        file.set_len(chunk_size).await?;
        let mut writer = BufWriter::with_capacity(chunk_size as usize, file);
        let mut limited_recv = recv.take(chunk_size);
        tokio::io::copy(&mut limited_recv, &mut writer).await?;
        writer.flush().await?;

        writer.into_inner().sync_all().await?;

        Ok(ChunkTransfer { data, offset: None })
    }
}

impl Drop for ChunkTransfer {
    fn drop(&mut self) {
        if self.data.exists() {
            let _ = std::fs::remove_file(&self.data);
        }
    }
}
