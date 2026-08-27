use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ToolType {
    Cli {
        command: String,
    },
    Webapp {
        url: String,
    },
    Api {
        base_url: String,
        auth_env: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(flatten)]
    pub tool_type: ToolType,
}

impl Tool {
    pub fn invocation(&self) -> String {
        match &self.tool_type {
            ToolType::Cli { command } => command.clone(),
            ToolType::Webapp { url } => url.clone(),
            ToolType::Api { base_url, .. } => base_url.clone(),
        }
    }

    pub fn type_label(&self) -> &'static str {
        match &self.tool_type {
            ToolType::Cli { .. } => "cli",
            ToolType::Webapp { .. } => "webapp",
            ToolType::Api { .. } => "api",
        }
    }
}
