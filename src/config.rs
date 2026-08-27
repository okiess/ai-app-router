use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::tool::Tool;

#[derive(Debug, Deserialize, Serialize)]
pub struct RawConfig {
    pub tool: BTreeMap<String, RawTool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawTool {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub tool_type: crate::tool::ToolType,
}

#[derive(Debug)]
pub struct Config {
    pub tools: Vec<Tool>,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => Self::discover_path().context("could not find config file")?,
        };

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config from {}", path.display()))?;

        let raw: RawConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse config from {}", path.display()))?;

        let mut tools = Vec::with_capacity(raw.tool.len());
        for (id, raw_tool) in raw.tool {
            tools.push(Tool {
                id,
                name: raw_tool.name,
                description: raw_tool.description,
                tags: raw_tool.tags,
                tool_type: raw_tool.tool_type,
            });
        }

        Ok(Config { tools })
    }

    pub fn discover_path() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from(".ai-app-router.toml"),
            dirs::config_dir()
                .map(|d| d.join("ai-app-router").join("config.toml"))
                .unwrap_or_default(),
            dirs::home_dir()
                .map(|d| d.join(".ai-app-router.toml"))
                .unwrap_or_default(),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }

        None
    }

    pub fn default_path() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("could not determine config directory")?
            .join("ai-app-router");
        Ok(dir.join("config.toml"))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }

        let mut tools = BTreeMap::<String, RawTool>::new();
        for tool in &self.tools {
            tools.insert(
                tool.id.clone(),
                RawTool {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    tags: tool.tags.clone(),
                    tool_type: tool.tool_type.clone(),
                },
            );
        }

        let raw = RawConfig { tool: tools };
        let content = toml::to_string_pretty(&raw).context("failed to serialize config to TOML")?;

        std::fs::write(path, content)
            .with_context(|| format!("failed to write config to {}", path.display()))?;

        Ok(())
    }
}
