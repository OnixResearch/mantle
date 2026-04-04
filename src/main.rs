use clap::Parser;
use snafu::prelude::*;
use tracing::info;

mod error;
use error::Result;

#[derive(Parser, Debug)]
#[command(name = "crunch", about = "crunch")]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[snafu::report]
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(if args.verbose {
                    tracing::Level::DEBUG.into()
                } else {
                    tracing::Level::INFO.into()
                }),
        )
        .init();

    info!("starting crunch");

    Ok(())
}
