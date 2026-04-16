use std::collections::HashMap;
use std::path::Path;

use petgraph::graph::{DiGraph, NodeIndex};

use crate::wiki::page::{PageType, WikiPage};

/// Relevance signal weights from research.md.
const DIRECT_LINK_WEIGHT: f64 = 3.0;
const SOURCE_OVERLAP_WEIGHT: f64 = 4.0;
const COMMON_NEIGHBORS_WEIGHT: f64 = 1.5;
const TYPE_AFFINITY_WEIGHT: f64 = 1.0;

/// Compute 4-signal relevance scores for all edges in the graph.
pub fn compute_relevance(
    graph: &mut DiGraph<String, f64>,
    node_indices: &HashMap<String, NodeIndex>,
    pages: &[WikiPage],
) {
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

    // Pre-compute neighbor sets for Adamic-Adar
    let neighbor_sets: HashMap<NodeIndex, Vec<NodeIndex>> = node_indices
        .values()
        .map(|&idx| {
            let neighbors: Vec<NodeIndex> = graph
                .neighbors_undirected(idx)
                .collect();
            (idx, neighbors)
        })
        .collect();

    // Update edge weights
    let edge_indices: Vec<_> = graph.edge_indices().collect();
    for edge_idx in edge_indices {
        if let Some((source_idx, target_idx)) = graph.edge_endpoints(edge_idx) {
            let source_slug = &graph[source_idx];
            let target_slug = &graph[target_idx];

            let source_page = page_map.get(source_slug.as_str());
            let target_page = page_map.get(target_slug.as_str());

            // Signal 1: Direct link (always present for edges)
            let direct = DIRECT_LINK_WEIGHT;

            // Signal 2: Source overlap
            let source_overlap = if let (Some(sp), Some(tp)) = (source_page, target_page) {
                let overlap = sp
                    .sources
                    .iter()
                    .filter(|s| tp.sources.contains(s))
                    .count();
                if overlap > 0 {
                    SOURCE_OVERLAP_WEIGHT * (overlap as f64).min(3.0)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Signal 3: Common neighbors (Adamic-Adar index)
            let common_neighbors = {
                let source_neighbors = neighbor_sets.get(&source_idx).cloned().unwrap_or_default();
                let target_neighbors = neighbor_sets.get(&target_idx).cloned().unwrap_or_default();

                let mut adamic_adar = 0.0_f64;
                for &common in &source_neighbors {
                    if target_neighbors.contains(&common) {
                        let degree = neighbor_sets
                            .get(&common)
                            .map(|n| n.len())
                            .unwrap_or(1);
                        if degree > 1 {
                            adamic_adar += 1.0 / (degree as f64).ln();
                        }
                    }
                }
                COMMON_NEIGHBORS_WEIGHT * adamic_adar
            };

            // Signal 4: Type affinity
            let type_affinity = if let (Some(sp), Some(tp)) = (source_page, target_page) {
                compute_type_affinity(&sp.page_type, &tp.page_type) * TYPE_AFFINITY_WEIGHT
            } else {
                0.0
            };

            let total_weight = direct + source_overlap + common_neighbors + type_affinity;
            graph[edge_idx] = total_weight;
        }
    }
}

/// Compute type affinity score between two page types.
fn compute_type_affinity(a: &PageType, b: &PageType) -> f64 {
    match (a, b) {
        // Entity ↔ Concept has highest affinity
        (PageType::Entity, PageType::Concept) | (PageType::Concept, PageType::Entity) => 1.0,
        // Source → Entity/Concept is natural
        (PageType::Source, PageType::Entity) | (PageType::Source, PageType::Concept) => 0.8,
        (PageType::Entity, PageType::Source) | (PageType::Concept, PageType::Source) => 0.8,
        // Same type has moderate affinity
        _ if a == b => 0.5,
        // Synthesis ↔ anything
        (PageType::Synthesis, _) | (_, PageType::Synthesis) => 0.6,
        _ => 0.3,
    }
}
