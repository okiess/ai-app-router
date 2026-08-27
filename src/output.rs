use anyhow::Result;
use comfy_table::{Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

use crate::resolver::Match;
use crate::tool::Tool;

pub enum OutputFormat {
    Human,
    Json,
}

impl PartialEq for OutputFormat {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (OutputFormat::Human, OutputFormat::Human) | (OutputFormat::Json, OutputFormat::Json)
        )
    }
}

impl OutputFormat {
    pub fn from_flag(json: bool) -> Self {
        if json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

pub fn list_tools(tools: &[Tool], format: OutputFormat, with_tags: bool) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(tools)?);
        }
        OutputFormat::Human => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["ID", "Name", "Type", "Description"]);

            if with_tags {
                table.set_header(vec!["ID", "Name", "Type", "Tags", "Description"]);
                for tool in tools {
                    table.add_row(vec![
                        &tool.id,
                        &tool.name,
                        tool.type_label(),
                        &tool.tags.join(", "),
                        &tool.description,
                    ]);
                }
            } else {
                for tool in tools {
                    table.add_row(vec![
                        &tool.id,
                        &tool.name,
                        tool.type_label(),
                        &tool.description,
                    ]);
                }
            }

            println!("{table}");
        }
    }
    Ok(())
}

pub fn show_tool(tool: &Tool, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(tool)?);
        }
        OutputFormat::Human => {
            println!("ID:          {}", tool.id);
            println!("Name:        {}", tool.name);
            println!("Type:        {}", tool.type_label());
            println!("Description: {}", tool.description);
            println!("Tags:        {}", tool.tags.join(", "));
            println!("Invocation:  {}", tool.invocation());
            if let crate::tool::ToolType::Api {
                auth_env: Some(env),
                ..
            } = &tool.tool_type
            {
                println!("Auth env:    {}", env);
            }
        }
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct ResolveJson<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    match_: Option<ResolveMatchJson>,
    candidates: Vec<ResolveMatchJson>,
}

#[derive(serde::Serialize)]
struct ResolveMatchJson {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_: String,
    description: String,
    invocation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_tag: Option<String>,
    confidence: f64,
}

impl ResolveMatchJson {
    fn from_match(m: &Match) -> Self {
        ResolveMatchJson {
            id: m.tool.id.clone(),
            name: m.tool.name.clone(),
            type_: m.tool.type_label().to_string(),
            description: m.tool.description.clone(),
            invocation: m.tool.invocation(),
            matched_tag: m.matched_tag.clone(),
            confidence: round_score(m.score),
        }
    }
}

pub fn resolve_output(query: &str, matches: &[Match], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let best = matches.first();
            let rest: Vec<&Match> = matches.iter().skip(1).collect();

            // Only treat as the unique "match" when confidence is meaningfully higher than the runner-up.
            let mut response = ResolveJson {
                query,
                match_: None,
                candidates: Vec::new(),
            };

            if let Some(best) = best {
                if rest.is_empty() || best.score - rest[0].score > 0.15 {
                    response.match_ = Some(ResolveMatchJson::from_match(best));
                    response.candidates = rest
                        .iter()
                        .map(|m| ResolveMatchJson::from_match(m))
                        .collect();
                } else {
                    response.candidates =
                        matches.iter().map(ResolveMatchJson::from_match).collect();
                }
            }

            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        OutputFormat::Human => {
            if matches.is_empty() {
                println!("No matching tool found for '{}'", query);
                return Ok(());
            }

            let best = &matches[0];
            println!("Best match for '{}':", query);
            println!("  ID:          {}", best.tool.id);
            println!("  Name:        {}", best.tool.name);
            println!("  Type:        {}", best.tool.type_label());
            println!("  Description: {}", best.tool.description);
            println!("  Invocation:  {}", best.tool.invocation());
            println!("  Confidence:  {:.2}", best.score);

            if matches.len() > 1 {
                println!("\nOther candidates:");
                for m in &matches[1..] {
                    println!(
                        "  - {} ({}) — confidence {:.2}",
                        m.tool.name,
                        m.tool.type_label(),
                        m.score
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn which_output(tool: &Tool) {
    println!("{}", tool.invocation());
}

fn round_score(score: f64) -> f64 {
    (score * 100.0).round() / 100.0
}
