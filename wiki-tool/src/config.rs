use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::WikiToolError;

/// Top-level configuration file structure (.wiki-tool.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub wiki: WikiConfig,
}

/// Wiki-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiConfig {
    /// Directory for raw source documents
    #[serde(default = "default_raw_dir")]
    pub raw_dir: String,
    /// Directory for wiki output
    #[serde(default = "default_wiki_dir")]
    pub wiki_dir: String,
    /// Directory for tool state
    #[serde(default = "default_state_dir")]
    pub state_dir: String,
}

impl Default for WikiConfig {
    fn default() -> Self {
        Self {
            raw_dir: default_raw_dir(),
            wiki_dir: default_wiki_dir(),
            state_dir: default_state_dir(),
        }
    }
}

fn default_raw_dir() -> String {
    "raw".to_string()
}
fn default_wiki_dir() -> String {
    "wiki".to_string()
}
fn default_state_dir() -> String {
    ".wiki-tool".to_string()
}

/// LLM provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    /// Active provider name (e.g., "anthropic", "openai", "ollama")
    #[serde(default)]
    pub provider: String,
    /// Provider-specific settings
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

/// Configuration for a specific LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API base URL
    pub api_url: String,
    /// API key (can also come from env var)
    #[serde(default)]
    pub api_key: Option<String>,
    /// Model name
    pub model: String,
    /// Max tokens for response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Request timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_max_tokens() -> u32 {
    8192
}
fn default_timeout_secs() -> u64 {
    900
}

impl AppConfig {
    /// Load config from a TOML file path.
    pub fn load(path: &Path) -> crate::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get the active LLM provider config, resolving API key from env vars.
    pub fn active_provider(&self) -> crate::Result<ProviderConfig> {
        let provider_name = &self.llm.provider;
        if provider_name.is_empty() {
            return Err(WikiToolError::Config(
                "No LLM provider configured. Set [llm] provider in .wiki-tool.toml".to_string(),
            ));
        }

        let mut config = self
            .llm
            .providers
            .get(provider_name)
            .ok_or_else(|| {
                WikiToolError::Config(format!("Provider '{}' not found in config", provider_name))
            })?
            .clone();

        // Resolve API key from environment if not set in config
        if config.api_key.is_none() {
            let env_var = match provider_name.as_str() {
                "anthropic" => "ANTHROPIC_API_KEY",
                "openai" => "OPENAI_API_KEY",
                "google" => "GOOGLE_API_KEY",
                _ => "",
            };
            if !env_var.is_empty() {
                if let Ok(key) = std::env::var(env_var) {
                    config.api_key = Some(key);
                }
            }
        }

        Ok(config)
    }

    /// Write a default config file.
    pub fn write_default(path: &Path) -> crate::Result<()> {
        let default_toml = r#"# wiki-tool configuration
# See quickstart.md for setup instructions

[wiki]
raw_dir = "raw"
wiki_dir = "wiki"
state_dir = ".wiki-tool"

[llm]
# Set the active provider (uncomment one):
# provider = "anthropic"
# provider = "openai"
# provider = "ollama"

[llm.providers.anthropic]
api_url = "https://api.anthropic.com/v1/messages"
model = "claude-sonnet-4-20250514"
max_tokens = 8192
timeout_secs = 900

[llm.providers.openai]
api_url = "https://api.openai.com/v1/chat/completions"
model = "gpt-4o"
max_tokens = 8192
timeout_secs = 900

[llm.providers.ollama]
api_url = "http://localhost:11434/api/chat"
model = "llama3"
max_tokens = 4096
timeout_secs = 300
"#;
        std::fs::write(path, default_toml)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            wiki: WikiConfig::default(),
        }
    }
}
