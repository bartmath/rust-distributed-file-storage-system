use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "client-cli", multicall = true)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: CliCommand,
}

/// Available client commands
#[derive(Subcommand, Debug)]
pub(super) enum CliCommand {
    /// List all files currently stored in the distributed file system.
    Ls,

    /// Upload a local file to the distributed storage.
    Upload {
        /// Path to the local file to upload.
        path: PathBuf,
    },

    /// Download a file from the distributed storage to the local machine.
    Download {
        /// The name of the file within the storage system.
        file_name: String,

        /// The local directory where the file should be saved.
        /// Defaults to the current working directory.
        #[arg(short = 'd', default_value = "./")]
        destination: PathBuf,
    },

    /// Terminate the client session and exit.
    Exit,
}
