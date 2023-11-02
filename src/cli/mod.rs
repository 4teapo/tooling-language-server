use clap::{Parser, Subcommand};

mod serve;
mod tracing;

use serve::*;
use tracing::*;

#[derive(Debug, Clone, Subcommand)]
pub enum CliSubcommand {
    Serve(ServeCommand),
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    subcommand: CliSubcommand,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub async fn run(self) {
        setup_tracing();

        match self.subcommand {
            CliSubcommand::Serve(cmd) => cmd.run().await,
        }
    }
}
