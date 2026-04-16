use std::path::Path;

use chrono::Local;

/// Operation types recorded in the log.
pub enum LogOperation {
    Ingest,
    Update,
    Delete,
    Rebuild,
    Query,
}

impl std::fmt::Display for LogOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ingest => write!(f, "ingest"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::Rebuild => write!(f, "rebuild"),
            Self::Query => write!(f, "query"),
        }
    }
}

/// Append an entry to log.md.
pub fn append_log(
    wiki_root: &Path,
    operation: LogOperation,
    details: &str,
) -> crate::Result<()> {
    let log_path = wiki_root.join("log.md");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    let entry = format!("- **[{}]** `{}` — {}\n", timestamp, operation, details);

    if log_path.exists() {
        let mut content = std::fs::read_to_string(&log_path)?;
        content.push_str(&entry);
        std::fs::write(&log_path, content)?;
    } else {
        let content = format!("# Wiki Log\n\n{}", entry);
        std::fs::write(&log_path, content)?;
    }

    Ok(())
}
