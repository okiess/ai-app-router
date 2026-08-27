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
            interactive,
        } => {
            let target_path = add_target_path(cli.config.as_deref())?;
            let mut config = Config::load(Some(&target_path)).unwrap_or(Config { tools: Vec::new() });

            let (id, name, description, tool_type, tags) = if interactive {
                interactive_tool_fields(
                    id.unwrap_or_default(),
                    name.unwrap_or_default(),
                    description.unwrap_or_default(),
                    type_.unwrap_or_default(),
                    command,
                    url,
                    base_url,
                    auth_env,
                    tags,
                )?
            } else {
                (
                    id.context("--id is required")?,
                    name.context("--name is required")?,
                    description.context("--description is required")?,
                    build_tool_type(
                        &type_.context("--type is required")?,
                        command.as_deref(),
                        url.as_deref(),
                        base_url.as_deref(),
                        auth_env.as_deref(),
                    )?,
                    tags,
                )
            };

            if !force && config.tools.iter().any(|t| t.id == id) {
                anyhow::bail!("tool '{}' already exists; use --force to overwrite", id);
            }

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

fn interactive_tool_fields(
    id: String,
    name: String,
    description: String,
    type_: String,
    command: Option<String>,
    url: Option<String>,
    base_url: Option<String>,
    auth_env: Option<String>,
    tags: Vec<String>,
) -> Result<(String, String, String, ai_app_router::tool::ToolType, Vec<String>)> {
    use dialoguer::{Confirm, Input, Select};

    let id = if id.is_empty() {
        Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Tool ID")
            .interact_text()?
    } else {
        id
    };

    let name = if name.is_empty() {
        Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Display name")
            .default(id.clone())
            .interact_text()?
    } else {
        name
    };

    let description = if description.is_empty() {
        Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Description")
            .interact_text()?
    } else {
        description
    };

    let type_options = vec!["cli", "webapp", "api"];
    let type_index = if type_.is_empty() {
        Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Tool type")
            .items(&type_options)
            .default(0)
            .interact()?
    } else {
        type_options
            .iter()
            .position(|t| *t == type_.to_lowercase())
            .unwrap_or(0)
    };
    let selected_type = type_options[type_index];

    let tool_type = match selected_type {
        "cli" => {
            let command = if let Some(c) = command {
                c
            } else {
                Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Command")
                    .interact_text()?
            };
            ai_app_router::tool::ToolType::Cli { command }
        }
        "webapp" => {
            let url = if let Some(u) = url {
                u
            } else {
                Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("URL")
                    .interact_text()?
            };
            ai_app_router::tool::ToolType::Webapp { url }
        }
        "api" => {
            let base_url = if let Some(u) = base_url {
                u
            } else {
                Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Base URL")
                    .interact_text()?
            };
            let auth_env = if let Some(e) = auth_env {
                Some(e)
            } else {
                let use_auth = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Use an environment variable for authentication?")
                    .default(false)
                    .interact()?;
                if use_auth {
                    Some(
                        Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt("Auth env variable name")
                            .interact_text()?,
                    )
                } else {
                    None
                }
            };
            ai_app_router::tool::ToolType::Api { base_url, auth_env }
        }
        _ => unreachable!(),
    };

    let tags = if tags.is_empty() {
        let raw: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Tags (comma-separated)")
            .allow_empty(true)
            .interact_text()?;
        raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    } else {
        tags
    };

    Ok((id, name, description, tool_type, tags))
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
