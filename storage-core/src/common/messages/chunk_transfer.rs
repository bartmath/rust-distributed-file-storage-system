use crate::common::config::TMP_STORAGE_ROOT;
use crate::common::types::ChunkId;
use quinn::{RecvStream, SendStream};
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};

#[derive(Debug, Default)]
pub struct ChunkTransfer {
    offset: Option<u64>,
    data_path: PathBuf,
    delete_on_error: bool,
}

impl ChunkTransfer {
    pub fn new(offset: Option<u64>, data_path: PathBuf, delete_on_error: bool) -> Self {
        ChunkTransfer {
            offset,
            data_path,
            delete_on_error,
        }
    }

    pub fn commit(mut self) -> PathBuf {
        self.delete_on_error = false;
        self.data_path.clone()
    }

    pub(crate) async fn send_chunk(
        &self,
        chunk_size: u64,
        send: &mut SendStream,
    ) -> anyhow::Result<()> {
        let mut file = File::open(&self.data_path).await?;

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

    pub(crate) async fn recv_chunk_server(
        chunk_id: ChunkId,
        chunk_size: u64,
        recv: &mut RecvStream,
    ) -> anyhow::Result<Self> {
        let data_path = TMP_STORAGE_ROOT
            .get()
            .expect("Temporary storage not initialized via config")
            .join(chunk_id.to_string());

        // We create the chunk_transfer before creating the file itself, to always drop it in case of any error.
        let chunk_transfer = ChunkTransfer::new(None, data_path.clone(), true);

        let file = File::create(&data_path).await?;
        file.set_len(chunk_size).await?;
        let mut writer = BufWriter::with_capacity(chunk_size as usize, file);
        let mut limited_recv = recv.take(chunk_size);
        tokio::io::copy(&mut limited_recv, &mut writer).await?;
        writer.flush().await?;

        writer.into_inner().sync_all().await?;

        Ok(chunk_transfer)
    }

    pub(crate) async fn recv_chunk_client(
        offset: u64,
        data_path: PathBuf,
        chunk_size: u64,
        recv: &mut RecvStream,
    ) -> anyhow::Result<Self> {
        let mut file = OpenOptions::new().write(true).open(&data_path).await?;

        let chunk_transfer = ChunkTransfer::new(Some(offset), data_path, false);

        file.seek(SeekFrom::Start(offset)).await?;

        let mut writer = BufWriter::with_capacity(chunk_size as usize, file);
        let mut limited_recv = recv.take(chunk_size);
        tokio::io::copy(&mut limited_recv, &mut writer).await?;
        writer.flush().await?;

        Ok(chunk_transfer)
    }
}

impl Drop for ChunkTransfer {
    fn drop(&mut self) {
        if self.data_path.exists() {
            let _ = std::fs::remove_file(&self.data_path);
        }
    }
}
