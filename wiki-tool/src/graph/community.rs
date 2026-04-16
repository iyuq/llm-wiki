use std::collections::HashMap;

use petgraph::graph::{DiGraph, NodeIndex};

/// Louvain community detection on a directed graph.
///
/// Iteratively assigns nodes to communities to maximize modularity.
/// Returns a mapping of NodeIndex → community_id.
pub fn detect_communities(graph: &DiGraph<String, f64>) -> HashMap<NodeIndex, u32> {
    let node_count = graph.node_count();
    if node_count == 0 {
        return HashMap::new();
    }

    // Initialize: each node in its own community
    let mut communities: HashMap<NodeIndex, u32> = HashMap::new();
    let nodes: Vec<NodeIndex> = graph.node_indices().collect();
    for (i, &node) in nodes.iter().enumerate() {
        communities.insert(node, i as u32);
    }

    let total_weight: f64 = graph
        .edge_indices()
        .map(|e| graph[e])
        .sum();

    if total_weight == 0.0 {
        return communities;
    }

    // Pre-compute node strengths (weighted degree)
    let mut node_strength: HashMap<NodeIndex, f64> = HashMap::new();
    for &node in &nodes {
        let strength: f64 = graph
            .edges(node)
            .map(|e| e.weight())
            .sum::<f64>()
            + graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .filter_map(|n| graph.find_edge(n, node))
                .map(|e| graph[e])
                .sum::<f64>();
        node_strength.insert(node, strength);
    }

    // Iterative optimization
    let max_iterations = 100;
    for _ in 0..max_iterations {
        let mut changed = false;

        for &node in &nodes {
            let current_community = communities[&node];

            // Calculate modularity gain for moving to each neighbor's community
            let neighbor_communities: Vec<u32> = graph
                .neighbors_undirected(node)
                .filter_map(|n| communities.get(&n).copied())
                .collect();

            let mut best_community = current_community;
            let mut best_gain = 0.0_f64;

            let mut tried: std::collections::HashSet<u32> = std::collections::HashSet::new();
            tried.insert(current_community);

            for &neighbor_comm in &neighbor_communities {
                if tried.contains(&neighbor_comm) {
                    continue;
                }
                tried.insert(neighbor_comm);

                let gain = modularity_gain(
                    graph,
                    node,
                    neighbor_comm,
                    &communities,
                    &node_strength,
                    total_weight,
                );

                if gain > best_gain {
                    best_gain = gain;
                    best_community = neighbor_comm;
                }
            }

            if best_community != current_community {
                communities.insert(node, best_community);
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    // Renumber communities sequentially from 0
    let mut community_remap: HashMap<u32, u32> = HashMap::new();
    let mut next_id = 0;
    for &node in &nodes {
        let comm = communities[&node];
        if !community_remap.contains_key(&comm) {
            community_remap.insert(comm, next_id);
            next_id += 1;
        }
    }

    for comm in communities.values_mut() {
        *comm = community_remap[comm];
    }

    communities
}

/// Calculate modularity gain for moving a node to a community.
fn modularity_gain(
    graph: &DiGraph<String, f64>,
    node: NodeIndex,
    target_community: u32,
    communities: &HashMap<NodeIndex, u32>,
    node_strength: &HashMap<NodeIndex, f64>,
    total_weight: f64,
) -> f64 {
    let ki = node_strength.get(&node).copied().unwrap_or(0.0);

    // Sum of weights of edges from node to nodes in target community
    let ki_in: f64 = graph
        .neighbors_undirected(node)
        .filter(|&n| communities.get(&n) == Some(&target_community))
        .map(|n| {
            let w1 = graph
                .find_edge(node, n)
                .map(|e| graph[e])
                .unwrap_or(0.0);
            let w2 = graph
                .find_edge(n, node)
                .map(|e| graph[e])
                .unwrap_or(0.0);
            w1 + w2
        })
        .sum();

    // Total weight of edges in target community
    let sigma_tot: f64 = graph
        .node_indices()
        .filter(|&n| communities.get(&n) == Some(&target_community) && n != node)
        .map(|n| node_strength.get(&n).copied().unwrap_or(0.0))
        .sum();

    let m2 = 2.0 * total_weight;
    if m2 == 0.0 {
        return 0.0;
    }

    // Modularity gain formula
    ki_in / m2 - (sigma_tot * ki) / (m2 * m2 / 4.0)
}
