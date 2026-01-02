use crate::client::Client;
use crate::commands::Cli;
use crate::config::ClientOpt;
use crate::setup::setup;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

mod client;
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
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let args = match shlex::split(line) {
                    Some(a) => a,
                    None => {
                        println!("Error: Invalid quoting in command");
                        continue;
                    }
                };

                let mut full_args = vec!["client-cli".to_string()];
                full_args.extend(args);

                match Cli::try_parse_from(full_args) {
                    Ok(cli) => {
                        if let Err(e) = client.handle_command(cli.command).await {
                            println!("Command failed: {}", e);
                        }
                    }
                    Err(e) => {
                        // Print Clap's error/help message
                        e.print().ok();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
