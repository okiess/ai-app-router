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
    let format = OutputFormat::from_flag(cli.json);

    match cli.command {
        Command::List { tags } => {
            let config = load_config(cli.config.as_deref())?;
            output::list_tools(&config.tools, format, tags)?;
        }
        Command::Show { id } => {
            let config = load_config(cli.config.as_deref())?;
            let tool = find_tool_by_id(&config.tools, &id)?;
            output::show_tool(tool, format)?;
        }
        Command::Resolve { query } => {
            let config = load_config(cli.config.as_deref())?;
            let query = query.join(" ");
            let matches = resolver::resolve(&query, &config.tools);
            output::resolve_output(&query, &matches, format)?;
        }
        Command::Which { query } => {
            let config = load_config(cli.config.as_deref())?;
            let query = query.join(" ");
            let matches = resolver::resolve(&query, &config.tools);
            let best = matches
                .first()
                .context("no matching tool found for the given task")?;
            output::which_output(&best.tool);
        }
        Command::Add {
            id,
            name,
            description,
            type_,
            command,
            url,
            base_url,
            auth_env,
            tags,
            force,
        } => {
            let target_path = add_target_path(cli.config.as_deref())?;
            let mut config = Config::load(Some(&target_path)).unwrap_or(Config { tools: Vec::new() });

            if !force && config.tools.iter().any(|t| t.id == id) {
                anyhow::bail!("tool '{}' already exists; use --force to overwrite", id);
            }

            let tool_type = build_tool_type(&type_, command.as_deref(), url.as_deref(), base_url.as_deref(), auth_env.as_deref())?;
            let tool = Tool {
                id: id.clone(),
                name,
                description,
                tags,
                tool_type,
            };

            config.tools.retain(|t| t.id != id);
            config.tools.push(tool);
            config.save(&target_path)?;

            if format == OutputFormat::Json {
                let added = find_tool_by_id(&config.tools, &id)?;
                output::show_tool(added, format)?;
            } else {
                println!("Added tool '{}' to {}", id, target_path.display());
            }
        }
    }

    Ok(())
}

fn load_config(path: Option<&std::path::Path>) -> Result<Config> {
    Config::load(path).context("failed to load configuration; create ~/.ai-app-router.toml or use --config")
}

fn add_target_path(cli_config: Option<&std::path::Path>) -> Result<std::path::PathBuf> {
    if let Some(path) = cli_config {
        return Ok(path.to_path_buf());
    }
    Config::discover_path()
        .map(Ok)
        .unwrap_or_else(|| Config::default_path())
}

fn build_tool_type(
    type_: &str,
    command: Option<&str>,
    url: Option<&str>,
    base_url: Option<&str>,
    auth_env: Option<&str>,
) -> Result<ai_app_router::tool::ToolType> {
    use ai_app_router::tool::ToolType;

    match type_.to_lowercase().as_str() {
        "cli" => {
            let command = command
                .context("--command is required for type=cli")?
                .to_string();
            Ok(ToolType::Cli { command })
        }
        "webapp" => {
            let url = url.context("--url is required for type=webapp")?.to_string();
            Ok(ToolType::Webapp { url })
        }
        "api" => {
            let base_url = base_url
                .context("--base-url is required for type=api")?
                .to_string();
            Ok(ToolType::Api {
                base_url,
                auth_env: auth_env.map(|s| s.to_string()),
            })
        }
        other => anyhow::bail!("unknown tool type '{}'", other),
    }
}

fn find_tool_by_id<'a>(tools: &'a [Tool], id: &'a str) -> Result<&'a Tool> {
    tools
        .iter()
        .find(|t| t.id == id)
        .with_context(|| format!("tool '{}' not found", id))
}
