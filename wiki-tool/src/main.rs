use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

/// LLM Wiki Agent — build and maintain a personal knowledge base from source documents.
#[derive(Parser)]
#[command(name = "wiki-tool", version, about, long_about = None)]
struct Cli {
    /// Config file path
    #[arg(short, long, default_value = ".wiki-tool.toml")]
    config: PathBuf,

    /// Project root directory
    #[arg(short, long, default_value = ".")]
    project: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long)]
    quiet: bool,

    /// Output as JSON (machine-readable)
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wiki project
    Init {
        /// Generate schema for specific agent(s)
        #[arg(long, default_value = "all")]
        schema: String,
    },

    /// Ingest a source document into the wiki (standalone mode)
    Ingest {
        /// Path to source file
        source_path: PathBuf,

        /// Re-ingest even if cached
        #[arg(long)]
        force: bool,

        /// Show what would be created without writing
        #[arg(long)]
        dry_run: bool,

        /// Disable streaming output
        #[arg(long)]
        no_stream: bool,
    },

    /// Ask a question and get a synthesized answer (standalone mode)
    Query {
        /// Natural language question
        question: String,

        /// Save answer as a synthesis page
        #[arg(long)]
        save: bool,

        /// Number of context pages to use
        #[arg(long, default_value = "5")]
        context: usize,

        /// Disable streaming output
        #[arg(long)]
        no_stream: bool,
    },

    /// Search wiki content by keyword
    Search {
        /// Search terms
        query: String,

        /// Max results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,

        /// Include content snippets
        #[arg(long)]
        snippet: bool,
    },

    /// Health-check the wiki for quality issues
    Lint {
        /// Auto-fix simple issues
        #[arg(long)]
        fix: bool,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },

    /// Build or update the knowledge graph
    Graph {
        /// Output file
        #[arg(short, long, default_value = "graph.json")]
        output: PathBuf,

        /// Output format: json or dot
        #[arg(long, default_value = "json")]
        format: String,

        /// Include Louvain community detection
        #[arg(long)]
        communities: bool,

        /// Show pages related to a specific page
        #[arg(long)]
        related: Option<String>,
    },

    /// Check or manage the ingest cache
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },

    /// Extract text content from a document
    Extract {
        /// Path to file
        file_path: PathBuf,
    },

    /// Rebuild wiki/index.md
    Index {
        /// Verify index is up-to-date (exit 1 if stale)
        #[arg(long)]
        check: bool,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Check if a source is cached
    Check {
        /// Path to source file
        source_path: PathBuf,
    },
    /// List all cached sources
    List,
    /// Clear cache entry
    Clear {
        /// Path to source file (or clear all if omitted)
        source_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let project_root = std::fs::canonicalize(&cli.project).unwrap_or_else(|_| cli.project.clone());
    let config_path = if cli.config.is_relative() {
        project_root.join(&cli.config)
    } else {
        cli.config.clone()
    };

    let config = wiki_tool::config::AppConfig::load(&config_path).unwrap_or_else(|e| {
        if cli.verbose {
            eprintln!("Warning: Could not load config: {}", e);
        }
        wiki_tool::config::AppConfig::default()
    });

    let ctx = commands::Context {
        config,
        project_root,
        verbose: cli.verbose,
        quiet: cli.quiet,
        json: cli.json,
    };

    let result = match cli.command {
        Commands::Init { schema } => commands::init::run(&ctx, &schema),
        Commands::Ingest {
            source_path,
            force,
            dry_run,
            no_stream,
        } => commands::ingest::run(&ctx, &source_path, force, dry_run, no_stream).await,
        Commands::Query {
            question,
            save,
            context,
            no_stream,
        } => commands::query::run(&ctx, &question, save, context, no_stream).await,
        Commands::Search {
            query,
            limit,
            snippet,
        } => commands::search::run(&ctx, &query, limit, snippet),
        Commands::Lint { fix, category } => commands::lint::run(&ctx, fix, category.as_deref()),
        Commands::Graph {
            output,
            format,
            communities,
            related,
        } => commands::graph::run(&ctx, &output, &format, communities, related.as_deref()),
        Commands::Cache { action } => match action {
            CacheAction::Check { source_path } => {
                commands::cache::run_check(&ctx, &source_path)
            }
            CacheAction::List => commands::cache::run_list(&ctx),
            CacheAction::Clear { source_path } => {
                commands::cache::run_clear(&ctx, source_path.as_deref())
            }
        },
        Commands::Extract { file_path } => commands::extract::run(&ctx, &file_path),
        Commands::Index { check } => commands::index::run(&ctx, check),
    };

    if let Err(e) = result {
        if cli.json {
            let err_json = serde_json::json!({ "error": e.to_string() });
            eprintln!("{}", err_json);
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}
