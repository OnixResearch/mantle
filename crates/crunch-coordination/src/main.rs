use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(
    name = "mantle-coordination",
    about = "Ephemeral local live-state coordination; never a store or evidence authority"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve bounded NDJSON subscriptions on a Unix socket.
    Serve {
        #[arg(long)]
        socket: PathBuf,
    },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    match Cli::parse().command {
        Command::Serve { socket } => crunch_coordination::serve(&socket).await,
    }
}
