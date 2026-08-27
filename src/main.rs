use anyhow::{Context, Result};
use clap::Parser;

use ai_app_router::cli::{Cli, Command};
use ai_app_router::config::Config;
use ai_app_router::output::{self, OutputFormat};
use ai_app_router::resolver;
use ai_app_router::tool::Tool;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())
        .context("failed to load configuration; create ~/.ai-app-router.toml or use --config")?;
    let format = OutputFormat::from_flag(cli.json);

    match cli.command {
        Command::List { tags } => {
            output::list_tools(&config.tools, format, tags)?;
        }
        Command::Show { id } => {
            let tool = find_tool_by_id(&config.tools, &id)?;
            output::show_tool(tool, format)?;
        }
        Command::Resolve { query } => {
            let query = query.join(" ");
            let matches = resolver::resolve(&query, &config.tools);
            output::resolve_output(&query, &matches, format)?;
        }
        Command::Which { query } => {
            let query = query.join(" ");
            let matches = resolver::resolve(&query, &config.tools);
            let best = matches
                .first()
                .context("no matching tool found for the given task")?;
            output::which_output(&best.tool);
        }
    }

    Ok(())
}

fn find_tool_by_id<'a>(tools: &'a [Tool], id: &'a str) -> Result<&'a Tool> {
    tools
        .iter()
        .find(|t| t.id == id)
        .with_context(|| format!("tool '{}' not found", id))
}
