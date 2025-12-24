use std::path::PathBuf;
use std::sync::OnceLock;

pub static TMP_STORAGE_ROOT: OnceLock<PathBuf> = OnceLock::new();
pub static FINAL_STORAGE_ROOT: OnceLock<PathBuf> = OnceLock::new();

pub const MAX_CHUNK_SIZE: usize = 1024 * 1024 * 64; // 64 MB
pub const N_CHUNK_REPLICAS: usize = 2;
pub const MAX_SPAWNED_TASKS: usize = 16;
