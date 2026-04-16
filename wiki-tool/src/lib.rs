pub mod cache;
pub mod config;
pub mod extract;
pub mod graph;
pub mod llm;
pub mod search;
pub mod wiki;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WikiToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Extraction error: {0}")]
    Extraction(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Wiki error: {0}")]
    Wiki(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, WikiToolError>;
