use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::WikiToolError;

/// Page type classification for wiki pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum PageType {
    Source,
    Entity,
    Concept,
    Synthesis,
}

impl fmt::Display for PageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source => write!(f, "source"),
            Self::Entity => write!(f, "entity"),
            Self::Concept => write!(f, "concept"),
            Self::Synthesis => write!(f, "synthesis"),
        }
    }
}

impl PageType {
    /// Returns the wiki subdirectory for this page type.
    pub fn subdirectory(&self) -> &'static str {
        match self {
            Self::Source => "sources",
            Self::Entity => "entities",
            Self::Concept => "concepts",
            Self::Synthesis => "syntheses",
        }
    }

    /// Parse from string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "source" => Some(Self::Source),
            "entity" => Some(Self::Entity),
            "concept" => Some(Self::Concept),
            "synthesis" => Some(Self::Synthesis),
            _ => None,
        }
    }
}

/// YAML frontmatter for a wiki page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    #[serde(rename = "type")]
    pub page_type: PageType,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub last_updated: String,
}

/// Represents a single wiki page (markdown file with YAML frontmatter).
#[derive(Debug, Clone)]
pub struct WikiPage {
    /// File path relative to wiki/ directory.
    pub path: String,
    /// Page title from frontmatter.
    pub title: String,
    /// Page type classification.
    pub page_type: PageType,
    /// Searchable tags.
    pub tags: Vec<String>,
    /// Source slugs that inform this page.
    pub sources: Vec<String>,
    /// ISO date of last update.
    pub last_updated: String,
    /// Raw markdown content (without frontmatter).
    pub content: String,
    /// Extracted [[wikilinks]] from content.
    pub wikilinks: Vec<String>,
}

impl WikiPage {
    /// Parse a WikiPage from markdown file content.
    pub fn from_markdown(path: &str, raw: &str) -> crate::Result<Self> {
        let (frontmatter, content) = Self::split_frontmatter(raw)?;
        let fm: Frontmatter =
            serde_yaml::from_str(&frontmatter).map_err(|e| WikiToolError::Yaml(e))?;
        let wikilinks = crate::wiki::wikilinks::extract_wikilinks(&content);

        Ok(Self {
            path: path.to_string(),
            title: fm.title,
            page_type: fm.page_type,
            tags: fm.tags,
            sources: fm.sources,
            last_updated: fm.last_updated,
            content,
            wikilinks,
        })
    }

    /// Split frontmatter from content. Frontmatter is delimited by `---`.
    fn split_frontmatter(raw: &str) -> crate::Result<(String, String)> {
        let trimmed = raw.trim_start();
        if !trimmed.starts_with("---") {
            return Err(WikiToolError::Parse(
                "Missing YAML frontmatter delimiter '---'".to_string(),
            ));
        }

        let after_first = &trimmed[3..];
        if let Some(end_idx) = after_first.find("\n---") {
            let frontmatter = after_first[..end_idx].trim().to_string();
            let content = after_first[end_idx + 4..].trim_start().to_string();
            Ok((frontmatter, content))
        } else {
            Err(WikiToolError::Parse(
                "Missing closing frontmatter delimiter '---'".to_string(),
            ))
        }
    }

    /// Serialize this page back to markdown with YAML frontmatter.
    pub fn to_markdown(&self) -> String {
        let fm = Frontmatter {
            title: self.title.clone(),
            page_type: self.page_type.clone(),
            tags: self.tags.clone(),
            sources: self.sources.clone(),
            last_updated: self.last_updated.clone(),
        };

        let yaml = serde_yaml::to_string(&fm).unwrap_or_default();
        format!("---\n{}---\n\n{}\n", yaml, self.content)
    }

    /// Read a wiki page from a file path.
    pub fn from_file(wiki_root: &Path, relative_path: &str) -> crate::Result<Self> {
        let full_path = wiki_root.join(relative_path);
        if !full_path.exists() {
            return Err(WikiToolError::FileNotFound(
                full_path.display().to_string(),
            ));
        }
        let raw = std::fs::read_to_string(&full_path)?;
        Self::from_markdown(relative_path, &raw)
    }

    /// Write this page to a file.
    pub fn write_to(&self, wiki_root: &Path) -> crate::Result<()> {
        let full_path = wiki_root.join(&self.path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, self.to_markdown())?;
        Ok(())
    }
}

/// Scan a wiki directory for all .md pages and parse them.
pub fn scan_wiki_pages(wiki_root: &Path) -> crate::Result<Vec<WikiPage>> {
    let mut pages = Vec::new();
    if !wiki_root.exists() {
        return Ok(pages);
    }
    scan_dir_recursive(wiki_root, wiki_root, &mut pages);
    Ok(pages)
}

fn scan_dir_recursive(base: &Path, dir: &Path, pages: &mut Vec<WikiPage>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(base, &path, pages);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            // Skip index.md and log.md at wiki root
            let relative = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if relative == "index.md" || relative == "log.md" || relative == "overview.md" {
                continue;
            }
            if let Ok(raw) = std::fs::read_to_string(&path) {
                if let Ok(page) = WikiPage::from_markdown(&relative, &raw) {
                    pages.push(page);
                }
            }
        }
    }
}
