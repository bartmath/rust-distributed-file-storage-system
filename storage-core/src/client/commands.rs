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
    /// List all files on the server
    Ls,
    /// Upload a file to the server
    Upload { path: PathBuf },
    /// Download a file from the server
    Download {
        /// The name of the remote file
        file_name: String,
        /// The local folder to save it to
        #[arg(short = 'd', default_value = "./")]
        destination: PathBuf,
    },
    /// Exit the client
    Exit,
}
