pub mod cache;
pub mod extract;
pub mod graph;
pub mod index;
pub mod ingest;
pub mod init;
pub mod lint;
pub mod query;
pub mod search;

use wiki_tool::config::AppConfig;
use std::path::PathBuf;

/// Shared context for all command handlers.
pub struct Context {
    pub config: AppConfig,
    pub project_root: PathBuf,
    pub verbose: bool,
    pub quiet: bool,
    pub json: bool,
}

impl Context {
    /// Get the wiki directory path.
    pub fn wiki_dir(&self) -> PathBuf {
        self.project_root.join(&self.config.wiki.wiki_dir)
    }

    /// Get the raw sources directory path.
    pub fn raw_dir(&self) -> PathBuf {
        self.project_root.join(&self.config.wiki.raw_dir)
    }

    /// Get the tool state directory path.
    pub fn state_dir(&self) -> PathBuf {
        self.project_root.join(&self.config.wiki.state_dir)
    }
}
