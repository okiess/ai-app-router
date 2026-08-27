use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::tool::Tool;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub tool: HashMap<String, RawTool>,
}

#[derive(Debug, Deserialize)]
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
}
