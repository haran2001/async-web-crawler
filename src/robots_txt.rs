// src/robots_txt.rs

use std::collections::HashMap;

/// Struct to hold robots.txt rules
#[derive(Debug, Default)]
pub struct RobotsTxt {
    /// Maps user-agent to their allowed and disallowed paths
    pub user_agents: HashMap<String, CrawlerRules>,
}

/// Struct to hold individual crawler rules
#[derive(Debug, Default)]
pub struct CrawlerRules {
    pub disallowed: Vec<String>,
    pub allowed: Vec<String>,
}

impl RobotsTxt {
    /// Parses the content of robots.txt and populates the `RobotsTxt` struct
    pub fn parse(&mut self, content: &str) {
        let mut current_agents: Vec<String> = Vec::new();

        for line in content.lines() {
            // Remove comments and trim whitespace
            let line = line.split('#').next().unwrap_or("").trim();

            if line.is_empty() {
                continue;
            }

            if let Some((field, value)) = line.split_once(':') {
                let field = field.trim().to_lowercase();
                let value = value.trim();

                match field.as_str() {
                    "user-agent" => {
                        current_agents.push(value.to_lowercase());
                        // Ensure there's an entry for this user-agent
                        self.user_agents
                            .entry(value.to_lowercase())
                            .or_insert_with(CrawlerRules::default);
                    }
                    "disallow" => {
                        for agent in &current_agents {
                            if let Some(rules) = self.user_agents.get_mut(agent) {
                                rules.disallowed.push(value.to_string());
                            }
                        }
                    }
                    "allow" => {
                        for agent in &current_agents {
                            if let Some(rules) = self.user_agents.get_mut(agent) {
                                rules.allowed.push(value.to_string());
                            }
                        }
                    }
                    _ => {
                        // Ignore other fields
                    }
                }
            }
        }
    }

    /// Determines if a path is allowed for the given user-agent
    pub fn is_allowed(&self, user_agent: &str, path: &str) -> bool {
        // Find rules for the most specific user-agent
        // First, try exact match
        let agent = self.user_agents.get(&user_agent.to_lowercase());

        // If exact match not found, try wildcard '*'
        let rules = agent.or_else(|| self.user_agents.get("*"));

        if let Some(rules) = rules {
            // Check Allow rules first
            for allow_path in &rules.allowed {
                if path.starts_with(allow_path) {
                    return true;
                }
            }

            // Then check Disallow rules
            for disallow_path in &rules.disallowed {
                if path.starts_with(disallow_path) {
                    return false;
                }
            }

            // If no matching rules, allow by default
            true
        } else {
            // If no rules for the user-agent, allow by default
            true
        }
    }
}
