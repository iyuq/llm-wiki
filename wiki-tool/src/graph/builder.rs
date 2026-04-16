use std::collections::HashMap;
use std::path::Path;

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

use crate::wiki::page::{scan_wiki_pages, WikiPage};
use crate::wiki::wikilinks::title_to_slug;

/// A node in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Wiki page identifier (filename without extension).
    pub id: String,
    /// Display title.
    pub title: String,
    /// Page type as string.
    #[serde(rename = "type")]
    pub node_type: String,
    /// Louvain community assignment.
    pub community: u32,
    /// Number of inbound + outbound links.
    pub links: u32,
}

/// An edge in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub weight: f64,
}

/// Full graph data structure for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub communities: Vec<CommunityInfo>,
    pub stats: GraphStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub id: u32,
    pub size: usize,
    pub cohesion: f64,
    pub top_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub nodes: usize,
    pub edges: usize,
    pub communities: usize,
}

/// Build a knowledge graph from all wiki pages.
pub fn build_graph(wiki_root: &Path) -> crate::Result<(DiGraph<String, f64>, HashMap<String, NodeIndex>, Vec<WikiPage>)> {
    let pages = scan_wiki_pages(wiki_root)?;
    let mut graph = DiGraph::<String, f64>::new();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Create nodes for all pages
    for page in &pages {
        let slug = Path::new(&page.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&page.path)
            .to_string();

        if !node_indices.contains_key(&slug) {
            let idx = graph.add_node(slug.clone());
            node_indices.insert(slug, idx);
        }
    }

    // Add edges from wikilinks
    for page in &pages {
        let source_slug = Path::new(&page.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&page.path)
            .to_string();

        let source_idx = match node_indices.get(&source_slug) {
            Some(idx) => *idx,
            None => continue,
        };

        for link in &page.wikilinks {
            let target_slug = title_to_slug(link);
            if let Some(&target_idx) = node_indices.get(&target_slug) {
                // Only add edge if it doesn't already exist
                if !graph.contains_edge(source_idx, target_idx) {
                    graph.add_edge(source_idx, target_idx, 1.0);
                }
            }
        }
    }

    Ok((graph, node_indices, pages))
}

/// Convert internal graph representation to serializable GraphData.
pub fn to_graph_data(
    graph: &DiGraph<String, f64>,
    node_indices: &HashMap<String, NodeIndex>,
    pages: &[WikiPage],
    communities: &HashMap<NodeIndex, u32>,
    community_info: Vec<CommunityInfo>,
) -> GraphData {
    let page_map: HashMap<String, &WikiPage> = pages
        .iter()
        .map(|p| {
            let slug = Path::new(&p.path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&p.path)
                .to_string();
            (slug, p)
        })
        .collect();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (slug, &idx) in node_indices {
        let page_type = page_map
            .get(slug)
            .map(|p| p.page_type.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let title = page_map
            .get(slug)
            .map(|p| p.title.clone())
            .unwrap_or_else(|| slug.clone());
        let community_id = communities.get(&idx).copied().unwrap_or(0);
        let link_count = graph.edges(idx).count() as u32
            + graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .count() as u32;

        nodes.push(GraphNode {
            id: slug.clone(),
            title,
            node_type: page_type,
            community: community_id,
            links: link_count,
        });
    }

    for edge in graph.edge_indices() {
        if let Some((source_idx, target_idx)) = graph.edge_endpoints(edge) {
            let weight = graph[edge];
            let source = graph[source_idx].clone();
            let target = graph[target_idx].clone();
            edges.push(GraphEdge {
                source,
                target,
                weight,
            });
        }
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| a.source.cmp(&b.source).then(a.target.cmp(&b.target)));

    let num_communities = if community_info.is_empty() {
        0
    } else {
        community_info.len()
    };

    GraphData {
        stats: GraphStats {
            nodes: nodes.len(),
            edges: edges.len(),
            communities: num_communities,
        },
        nodes,
        edges,
        communities: community_info,
    }
}
