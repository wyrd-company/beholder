// ---
// relationships:
//   implements: compiler-backed-code-graph
// ---

//! Reproducible coupling views derived only from stored nodes and edges.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::model::CodeGraph;
use crate::query::{DependencyPath, Direction, GraphQuery};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouplingReport {
    pub degree: BTreeMap<String, Degree>,
    pub cycles: Vec<Region>,
    pub weak_regions: Vec<Region>,
    pub bridges: Vec<BridgePosition>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Degree {
    pub incoming_neighbours: Vec<String>,
    pub outgoing_neighbours: Vec<String>,
    pub incoming_reach: Vec<DependencyPath>,
    pub outgoing_reach: Vec<DependencyPath>,
    pub evidence_edges: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub nodes: Vec<String>,
    pub evidence_edges: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgePosition {
    pub node: String,
    pub regions_before: usize,
    pub regions_after_removal: usize,
    pub neighbour_partitions: Vec<Vec<String>>,
    pub evidence_edges: Vec<String>,
}

pub fn analyze(graph: &CodeGraph) -> CouplingReport {
    let adjacency = adjacency(graph);
    let reverse = reverse_adjacency(graph);
    let degree = degrees(graph, &adjacency, &reverse);
    let cycles = cycles(graph, &adjacency);
    let regions = weak_regions(graph, None);
    let regions_before = regions.len();
    let query = GraphQuery::new(graph);
    let bridges = graph
        .nodes
        .iter()
        .filter_map(|node| {
            let after = weak_regions(graph, Some(&node.id));
            let neighbours = adjacency[&node.id.as_str()]
                .union(&reverse[&node.id.as_str()])
                .copied()
                .collect::<BTreeSet<_>>();
            (after.len() > regions_before).then(|| BridgePosition {
                node: node.id.clone(),
                regions_before,
                regions_after_removal: after.len(),
                neighbour_partitions: after
                    .iter()
                    .map(|region| {
                        region
                            .nodes
                            .iter()
                            .filter(|candidate| neighbours.contains(candidate.as_str()))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .filter(|partition| !partition.is_empty())
                    .collect(),
                evidence_edges: graph
                    .edges
                    .iter()
                    .filter(|edge| edge.from == node.id || edge.to == node.id)
                    .map(|edge| edge.id.clone())
                    .collect(),
            })
        })
        .collect();

    CouplingReport {
        degree: degree
            .into_iter()
            .map(|(node, mut degree)| {
                degree.incoming_reach = query.reachable(&node, Direction::Dependants);
                degree.outgoing_reach = query.reachable(&node, Direction::Dependencies);
                (node, degree)
            })
            .collect(),
        cycles,
        weak_regions: regions,
        bridges,
    }
}

fn adjacency(graph: &CodeGraph) -> BTreeMap<&str, BTreeSet<&str>> {
    let mut result = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        result
            .entry(edge.from.as_str())
            .or_default()
            .insert(edge.to.as_str());
    }
    result
}

fn reverse_adjacency(graph: &CodeGraph) -> BTreeMap<&str, BTreeSet<&str>> {
    let mut result = graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        result
            .entry(edge.to.as_str())
            .or_default()
            .insert(edge.from.as_str());
    }
    result
}

fn degrees(
    graph: &CodeGraph,
    outgoing: &BTreeMap<&str, BTreeSet<&str>>,
    incoming: &BTreeMap<&str, BTreeSet<&str>>,
) -> BTreeMap<String, Degree> {
    graph
        .nodes
        .iter()
        .map(|node| {
            let mut evidence_edges = graph
                .edges
                .iter()
                .filter(|edge| edge.from == node.id || edge.to == node.id)
                .map(|edge| edge.id.clone())
                .collect::<Vec<_>>();
            evidence_edges.sort();
            evidence_edges.dedup();
            (
                node.id.clone(),
                Degree {
                    incoming_neighbours: incoming[&node.id.as_str()]
                        .iter()
                        .filter(|neighbour| **neighbour != node.id)
                        .map(|neighbour| (*neighbour).to_owned())
                        .collect(),
                    outgoing_neighbours: outgoing[&node.id.as_str()]
                        .iter()
                        .filter(|neighbour| **neighbour != node.id)
                        .map(|neighbour| (*neighbour).to_owned())
                        .collect(),
                    incoming_reach: Vec::new(),
                    outgoing_reach: Vec::new(),
                    evidence_edges,
                },
            )
        })
        .collect()
}

fn cycles(graph: &CodeGraph, adjacency: &BTreeMap<&str, BTreeSet<&str>>) -> Vec<Region> {
    struct Tarjan<'a> {
        next_index: usize,
        index: BTreeMap<&'a str, usize>,
        lowlink: BTreeMap<&'a str, usize>,
        stack: Vec<&'a str>,
        on_stack: BTreeSet<&'a str>,
        components: Vec<Vec<&'a str>>,
    }

    fn visit<'a>(
        node: &'a str,
        graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
        state: &mut Tarjan<'a>,
    ) {
        let index = state.next_index;
        state.next_index += 1;
        state.index.insert(node, index);
        state.lowlink.insert(node, index);
        state.stack.push(node);
        state.on_stack.insert(node);

        for &next in &graph[node] {
            if !state.index.contains_key(next) {
                visit(next, graph, state);
                state
                    .lowlink
                    .insert(node, state.lowlink[node].min(state.lowlink[next]));
            } else if state.on_stack.contains(next) {
                state
                    .lowlink
                    .insert(node, state.lowlink[node].min(state.index[next]));
            }
        }

        if state.lowlink[node] == state.index[node] {
            let mut component = Vec::new();
            while let Some(member) = state.stack.pop() {
                state.on_stack.remove(member);
                component.push(member);
                if member == node {
                    break;
                }
            }
            component.sort();
            state.components.push(component);
        }
    }

    let mut state = Tarjan {
        next_index: 0,
        index: BTreeMap::new(),
        lowlink: BTreeMap::new(),
        stack: Vec::new(),
        on_stack: BTreeSet::new(),
        components: Vec::new(),
    };
    for &node in adjacency.keys() {
        if !state.index.contains_key(node) {
            visit(node, adjacency, &mut state);
        }
    }

    let mut result = state
        .components
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || component
                    .first()
                    .is_some_and(|node| adjacency[node].contains(node))
        })
        .map(|nodes| region(graph, nodes.into_iter().map(str::to_owned).collect()))
        .collect::<Vec<_>>();
    result.sort_by(|left, right| left.nodes.cmp(&right.nodes));
    result
}

fn weak_regions(graph: &CodeGraph, removed: Option<&str>) -> Vec<Region> {
    let mut neighbours = graph
        .nodes
        .iter()
        .filter(|node| Some(node.id.as_str()) != removed)
        .map(|node| (node.id.as_str(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        if Some(edge.from.as_str()) == removed || Some(edge.to.as_str()) == removed {
            continue;
        }
        if neighbours.contains_key(edge.from.as_str()) && neighbours.contains_key(edge.to.as_str())
        {
            neighbours.entry(&edge.from).or_default().insert(&edge.to);
            neighbours.entry(&edge.to).or_default().insert(&edge.from);
        }
    }

    let mut visited = BTreeSet::new();
    let mut regions = Vec::new();
    for &start in neighbours.keys() {
        if !visited.insert(start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut nodes = Vec::new();
        while let Some(node) = queue.pop_front() {
            nodes.push(node.to_owned());
            for &next in &neighbours[node] {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        nodes.sort();
        regions.push(region(graph, nodes));
    }
    regions.sort_by(|left, right| left.nodes.cmp(&right.nodes));
    regions
}

fn region(graph: &CodeGraph, nodes: Vec<String>) -> Region {
    let members = nodes.iter().map(String::as_str).collect::<BTreeSet<_>>();
    Region {
        evidence_edges: graph
            .edges
            .iter()
            .filter(|edge| {
                members.contains(edge.from.as_str()) && members.contains(edge.to.as_str())
            })
            .map(|edge| edge.id.clone())
            .collect(),
        nodes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::graph;

    #[test]
    fn distinct_degrees_ignore_parallel_edges_and_self_edges() {
        let mut graph = graph(&[("amber", "birch"), ("amber", "birch"), ("amber", "amber")]);
        for (index, edge) in graph.edges.iter_mut().enumerate() {
            edge.id = format!("edge-{index}");
        }
        let report = analyze(&graph);
        assert_eq!(report.degree["amber"].outgoing_neighbours, ["birch"]);
    }

    #[test]
    fn strongly_connected_cycles_are_reported_with_evidence() {
        let graph = graph(&[("amber", "birch"), ("birch", "cedar"), ("cedar", "amber")]);
        let report = analyze(&graph);
        assert_eq!(report.cycles[0].nodes, ["amber", "birch", "cedar"]);
        assert_eq!(report.cycles[0].evidence_edges.len(), 3);
    }

    #[test]
    fn weak_regions_ignore_edge_direction_and_include_isolates() {
        let graph = graph(&[("birch", "amber"), ("cedar", "cedar")]);
        let report = analyze(&graph);
        assert_eq!(report.weak_regions.len(), 2);
    }

    #[test]
    fn bridge_position_is_proved_by_node_removal() {
        let graph = graph(&[("amber", "birch"), ("birch", "cedar")]);
        let report = analyze(&graph);
        assert_eq!(report.bridges.len(), 1);
        assert_eq!(report.bridges[0].node, "birch");
        assert_eq!(report.bridges[0].regions_before, 1);
        assert_eq!(report.bridges[0].regions_after_removal, 2);
        assert_eq!(
            report.bridges[0].neighbour_partitions,
            [vec!["amber".to_owned()], vec!["cedar".to_owned()]]
        );
    }

    #[test]
    fn coupling_carries_transitive_incoming_and_outgoing_paths() {
        let graph = graph(&[("amber", "birch"), ("birch", "cedar")]);
        let report = analyze(&graph);

        assert_eq!(report.degree["amber"].outgoing_reach.len(), 2);
        assert_eq!(report.degree["cedar"].incoming_reach.len(), 2);
        assert_eq!(
            report.degree["amber"].outgoing_reach[1].nodes,
            ["amber", "birch", "cedar"]
        );
    }
}
