use std::cmp::Ordering;

use strsim::jaro_winkler;

use crate::tool::Tool;

pub const SCORE_EXACT: f64 = 1.0;
pub const SCORE_SUBSTRING: f64 = 0.8;
pub const SCORE_TOKEN_OVERLAP: f64 = 0.5;
pub const SCORE_FUZZY: f64 = 0.3;
pub const SCORE_THRESHOLD: f64 = 0.2;

#[derive(Debug, Clone)]
pub struct Match {
    pub tool: Tool,
    pub score: f64,
    pub matched_tag: Option<String>,
}

pub fn resolve(query: &str, tools: &[Tool]) -> Vec<Match> {
    let query_lower = query.to_lowercase();
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<Match> = tools
        .iter()
        .map(|tool| score_tool(&query_lower, &query_tokens, tool))
        .filter(|m| m.score >= SCORE_THRESHOLD)
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.tool.name.cmp(&b.tool.name))
    });

    scored
}

fn score_tool(query: &str, query_tokens: &[&str], tool: &Tool) -> Match {
    let mut best_score = 0.0;
    let mut matched_tag: Option<String> = None;

    // Exact tag match
    for tag in &tool.tags {
        let tag_lower = tag.to_lowercase();
        if tag_lower == query {
            return Match {
                tool: tool.clone(),
                score: SCORE_EXACT,
                matched_tag: Some(tag.clone()),
            };
        }

        // Substring match in tag
        if tag_lower.contains(query) || query.contains(&tag_lower) {
            let score = SCORE_SUBSTRING;
            if score > best_score {
                best_score = score;
                matched_tag = Some(tag.clone());
            }
        }
    }

    // Token overlap
    let token_score = token_overlap_score(query_tokens, tool);
    if token_score > best_score {
        best_score = token_score;
        matched_tag = None;
    }

    // Name / description fuzzy similarity
    let name_lower = tool.name.to_lowercase();
    let desc_lower = tool.description.to_lowercase();

    let name_sim = jaro_winkler(&name_lower, query);
    let desc_sim = jaro_winkler(&desc_lower, query);
    let fuzzy = name_sim.max(desc_sim) * SCORE_FUZZY;

    if fuzzy > best_score {
        best_score = fuzzy;
        matched_tag = None;
    }

    Match {
        tool: tool.clone(),
        score: best_score,
        matched_tag,
    }
}

fn token_overlap_score(query_tokens: &[&str], tool: &Tool) -> f64 {
    if query_tokens.is_empty() {
        return 0.0;
    }

    let haystacks: Vec<String> = tool
        .tags
        .iter()
        .map(|t| t.to_lowercase())
        .chain(std::iter::once(tool.name.to_lowercase()))
        .chain(std::iter::once(tool.description.to_lowercase()))
        .collect();

    let mut total_matches = 0usize;
    for token in query_tokens {
        let token_len = token.len();
        if token_len < 2 {
            continue;
        }
        for hay in &haystacks {
            if hay.contains(token) {
                total_matches += 1;
                break;
            }
        }
    }

    if total_matches == 0 {
        return 0.0;
    }

    let ratio = total_matches as f64 / query_tokens.len() as f64;
    SCORE_TOKEN_OVERLAP * ratio
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{Tool, ToolType};

    fn tool(id: &str, name: &str, tags: &[&str]) -> Tool {
        Tool {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{} tool", name),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            tool_type: ToolType::Cli {
                command: id.to_string(),
            },
        }
    }

    #[test]
    fn exact_tag_match_is_top() {
        let tools = vec![
            tool("fltw", "fltw", &["time tracking", "freelance"]),
            tool("mvapi", "mvapi", &["markdown vault", "brain"]),
        ];

        let result = resolve("time tracking", &tools);
        assert_eq!(result[0].tool.id, "fltw");
        assert!((result[0].score - SCORE_EXACT).abs() < f64::EPSILON);
    }

    #[test]
    fn substring_and_fuzzy_work() {
        let tools = vec![tool("dividends-cli", "dividends-cli", &["dividends"])];
        let result = resolve("dividenden", &tools);
        assert_eq!(result[0].tool.id, "dividends-cli");
    }
}
