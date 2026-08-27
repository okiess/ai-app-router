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

    /// Add a new tool to the configuration
    Add {
        /// Unique tool identifier
        #[arg(long)]
        id: String,

        /// Display name
        #[arg(long)]
        name: String,

        /// Short description
        #[arg(long)]
        description: String,

        /// Tool type: cli, webapp, or api
        #[arg(long, value_name = "TYPE")]
        type_: String,

        /// Command to invoke (required for type=cli)
        #[arg(long)]
        command: Option<String>,

        /// URL to open (required for type=webapp)
        #[arg(long)]
        url: Option<String>,

        /// API base URL (required for type=api)
        #[arg(long)]
        base_url: Option<String>,

        /// Environment variable holding the API key (only for type=api)
        #[arg(long)]
        auth_env: Option<String>,

        /// Comma-separated list of tags
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Overwrite an existing tool with the same ID
        #[arg(long)]
        force: bool,

        /// Run interactive prompts for all fields
        #[arg(short, long)]
        interactive: bool,
    },
}
