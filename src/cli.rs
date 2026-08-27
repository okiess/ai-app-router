use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "aiar", version, about = "AI App Router — map tasks to tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Path to a custom config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Output JSON instead of human-readable text
    #[arg(short, long, global = true)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List all configured tools
    List {
        /// Include tags in the output
        #[arg(long)]
        tags: bool,
    },

    /// Show details for a specific tool
    Show { id: String },

    /// Resolve a task to the best matching tool
    Resolve { query: Vec<String> },

    /// Print only the invocation for the best matching tool
    Which { query: Vec<String> },
}
