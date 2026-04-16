use std::collections::HashMap;
use std::path::Path;

use super::Context;
use wiki_tool::graph::builder::{build_graph, to_graph_data, CommunityInfo};
use wiki_tool::graph::community::detect_communities;
use wiki_tool::graph::relevance::compute_relevance;
use wiki_tool::wiki::wikilinks::title_to_slug;

/// Run the graph command.
pub fn run(
    ctx: &Context,
    output: &Path,
    format: &str,
    communities: bool,
    related: Option<&str>,
) -> wiki_tool::Result<()> {
    let wiki_dir = ctx.wiki_dir();

    if !wiki_dir.exists() {
        return Err(wiki_tool::WikiToolError::Wiki(
            "Wiki directory not found. Run 'wiki-tool init' first.".to_string(),
        ));
    }

    let (mut graph, node_indices, pages) = build_graph(&wiki_dir)?;

    if !ctx.quiet && !ctx.json {
        println!("📊 Building knowledge graph...");
    }

    // Compute relevance scores
    compute_relevance(&mut graph, &node_indices, &pages);

    // Run community detection if requested
    let community_map = if communities {
        if !ctx.quiet && !ctx.json {
            println!("  Running Louvain community detection...");
        }
        detect_communities(&graph)
    } else {
        HashMap::new()
    };

    // Build community info
    let community_info = if communities {
        let mut comm_nodes: HashMap<u32, Vec<String>> = HashMap::new();
        for (slug, &idx) in &node_indices {
            let comm_id = community_map.get(&idx).copied().unwrap_or(0);
            comm_nodes.entry(comm_id).or_default().push(slug.clone());
        }

        let mut info: Vec<CommunityInfo> = comm_nodes
            .into_iter()
            .map(|(id, mut members)| {
                members.sort();
                let size = members.len();
                let top_nodes: Vec<String> = members.into_iter().take(5).collect();
                CommunityInfo {
                    id,
                    size,
                    cohesion: 0.0, // Simplified; full cohesion calculation omitted
                    top_nodes,
                }
            })
            .collect();
        info.sort_by_key(|c| c.id);
        info
    } else {
        Vec::new()
    };

    // Handle --related <PAGE>
    if let Some(page_name) = related {
        let slug = title_to_slug(page_name);
        if let Some(&idx) = node_indices.get(&slug) {
            let neighbors: Vec<String> = graph
                .neighbors_undirected(idx)
                .filter_map(|n| Some(graph[n].clone()))
                .collect();

            if ctx.json {
                let result = serde_json::json!({
                    "page": slug,
                    "related": neighbors,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Pages related to '{}':", page_name);
                for n in &neighbors {
                    println!("  - {}", n);
                }
            }
            return Ok(());
        } else {
            return Err(wiki_tool::WikiToolError::Wiki(format!(
                "Page '{}' not found in graph",
                page_name
            )));
        }
    }

    let graph_data = to_graph_data(&graph, &node_indices, &pages, &community_map, community_info);

    // Write output
    let output_path = if output.is_relative() {
        ctx.project_root.join(output)
    } else {
        output.to_path_buf()
    };

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&graph_data)?;
            std::fs::write(&output_path, &json)?;
        }
        "dot" => {
            let dot = generate_dot(&graph_data);
            std::fs::write(&output_path, &dot)?;
        }
        _ => {
            return Err(wiki_tool::WikiToolError::Config(format!(
                "Unknown format: {}. Use 'json' or 'dot'.",
                format
            )));
        }
    }

    // Report
    if ctx.json {
        println!("{}", serde_json::to_string_pretty(&graph_data)?);
    } else if !ctx.quiet {
        println!(
            "\n✓ Graph written to {} ({} nodes, {} edges",
            output_path.display(),
            graph_data.stats.nodes,
            graph_data.stats.edges,
        );
        if communities {
            println!(", {} communities)", graph_data.stats.communities);
        } else {
            println!(")");
        }
    }

    Ok(())
}

/// Generate DOT format for Graphviz visualization.
fn generate_dot(data: &wiki_tool::graph::builder::GraphData) -> String {
    let mut dot = String::from("digraph wiki {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box];\n\n");

    for node in &data.nodes {
        let color = match node.node_type.as_str() {
            "source" => "lightblue",
            "entity" => "lightgreen",
            "concept" => "lightyellow",
            "synthesis" => "lightpink",
            _ => "white",
        };
        dot.push_str(&format!(
            "  \"{}\" [label=\"{}\" fillcolor=\"{}\" style=filled];\n",
            node.id, node.title, color
        ));
    }

    dot.push('\n');

    for edge in &data.edges {
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{:.1}\"];\n",
            edge.source, edge.target, edge.weight
        ));
    }

    dot.push_str("}\n");
    dot
}
