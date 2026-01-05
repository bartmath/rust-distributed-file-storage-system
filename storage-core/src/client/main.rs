use crate::client::{Client, LoopAction};
use crate::commands::Cli;
use crate::config::ClientOpt;
use crate::setup::setup;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

mod chunkserver_connection_pool;
mod client;
mod client_chunk_uploader;
mod commands;
mod config;
mod setup;
mod types;

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let opt = ClientOpt::parse();
    let client = setup(opt).expect("Couldn't set the client up");
    run(client).await.expect("Client error");
}

async fn run(client: Client) -> anyhow::Result<()> {
    let mut rl = DefaultEditor::new()?;

    println!("Welcome to Distributed file storage system! Type 'help' for commands.");

    loop {
        let readline = rl.readline(">> ");
        let args = match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match shlex::split(line) {
                    Some(a) => a,
                    None => {
                        println!("Error: Invalid quoting in command");
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        match Cli::try_parse_from(args) {
            Ok(cli) => match client.handle_command(cli.command).await {
                Ok(LoopAction::Continue) => {}
                Ok(LoopAction::Exit) => break,
                Err(e) => println!("Command error: {}", e),
            },
            Err(e) => {
                // Print Clap's error/help message
                e.print().ok();
            }
        };
    }

    println!("Closing session...");
    let _ = tokio::time::timeout(std::time::Duration::from_secs(1), client.close_session()).await;

    Ok(())
}
