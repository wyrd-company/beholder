// ---
// relationships:
//   implements: compiler-backed-code-graph
// ---

//! Deterministic dependency paths over the language-neutral graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::model::{CodeGraph, Edge, EvidencePolicy, UncertaintyCounts};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Dependencies,
    Dependants,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DependencyPath {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct QueryReport {
    pub schema_version: u32,
    pub snapshot_id: String,
    pub evidence_policy: EvidencePolicy,
    pub uncertainty: UncertaintyCounts,
    pub start: String,
    pub direction: Direction,
    pub transitive: bool,
    pub paths: Vec<DependencyPath>,
}

pub struct GraphQuery<'a> {
    graph: &'a CodeGraph,
    nodes: BTreeSet<&'a str>,
    outgoing: BTreeMap<&'a str, Vec<&'a Edge>>,
    incoming: BTreeMap<&'a str, Vec<&'a Edge>>,
}

impl<'a> GraphQuery<'a> {
    pub fn new(graph: &'a CodeGraph) -> Self {
        let mut outgoing: BTreeMap<&str, Vec<&Edge>> = BTreeMap::new();
        let mut incoming: BTreeMap<&str, Vec<&Edge>> = BTreeMap::new();
        for edge in &graph.edges {
            outgoing.entry(&edge.from).or_default().push(edge);
            incoming.entry(&edge.to).or_default().push(edge);
        }
        for edges in outgoing.values_mut().chain(incoming.values_mut()) {
            edges.sort_by(|left, right| {
                (&left.from, &left.to, &left.kind, &left.id).cmp(&(
                    &right.from,
                    &right.to,
                    &right.kind,
                    &right.id,
                ))
            });
        }
        Self {
            graph,
            nodes: graph.nodes.iter().map(|node| node.id.as_str()).collect(),
            outgoing,
            incoming,
        }
    }

    pub fn direct(&self, node: &str, direction: Direction) -> Option<Vec<DependencyPath>> {
        self.nodes.contains(node).then(|| {
            self.steps(node, direction)
                .into_iter()
                .map(|(next, edge)| DependencyPath {
                    nodes: vec![node.to_owned(), next.to_owned()],
                    edges: vec![edge.id.clone()],
                })
                .collect()
        })
    }

    /// One deterministic shortest evidence path for every reachable node.
    pub fn reachable(&self, start: &str, direction: Direction) -> Option<Vec<DependencyPath>> {
        if !self.nodes.contains(start) {
            return None;
        }
        let mut queue = VecDeque::from([start.to_owned()]);
        let mut visited = BTreeSet::from([start.to_owned()]);
        let mut paths = BTreeMap::<String, DependencyPath>::new();

        while let Some(current) = queue.pop_front() {
            let prefix = paths.get(&current).cloned().unwrap_or(DependencyPath {
                nodes: vec![start.to_owned()],
                edges: Vec::new(),
            });
            for (next, edge) in self.steps(&current, direction) {
                if visited.insert(next.to_owned()) {
                    let mut path = prefix.clone();
                    path.nodes.push(next.to_owned());
                    path.edges.push(edge.id.clone());
                    paths.insert(next.to_owned(), path);
                    queue.push_back(next.to_owned());
                }
            }
        }

        Some(paths.into_values().collect())
    }

    pub fn report(
        &self,
        start: &str,
        direction: Direction,
        transitive: bool,
    ) -> Option<QueryReport> {
        let paths = if transitive {
            self.reachable(start, direction)
        } else {
            self.direct(start, direction)
        }?;
        Some(QueryReport {
            schema_version: self.graph.schema_version,
            snapshot_id: self.graph.build.id.clone(),
            evidence_policy: EvidencePolicy::all_stored(self.graph),
            uncertainty: UncertaintyCounts::from(self.graph),
            start: start.to_owned(),
            direction,
            transitive,
            paths,
        })
    }

    fn steps(&self, node: &str, direction: Direction) -> Vec<(&'a str, &'a Edge)> {
        let edges = match direction {
            Direction::Dependencies => self.outgoing.get(node),
            Direction::Dependants => self.incoming.get(node),
        };
        let mut steps = edges
            .into_iter()
            .flatten()
            .map(|edge| match direction {
                Direction::Dependencies => (edge.to.as_str(), *edge),
                Direction::Dependants => (edge.from.as_str(), *edge),
            })
            .collect::<Vec<_>>();
        steps.sort_by(|left, right| (left.0, &left.1.id).cmp(&(right.0, &right.1.id)));
        steps
    }

    pub fn graph(&self) -> &'a CodeGraph {
        self.graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::graph;

    #[test]
    fn direct_dependency_and_dependant_directions_are_opposites() {
        let graph = graph(&[("amber", "birch"), ("cedar", "amber")]);
        let query = GraphQuery::new(&graph);

        assert_eq!(
            query.direct("amber", Direction::Dependencies).unwrap()[0].nodes,
            ["amber", "birch"]
        );
        assert_eq!(
            query.direct("amber", Direction::Dependants).unwrap()[0].nodes,
            ["amber", "cedar"]
        );
    }

    #[test]
    fn transitive_reach_uses_a_deterministic_shortest_path() {
        let graph = graph(&[
            ("amber", "cedar"),
            ("amber", "birch"),
            ("birch", "elm"),
            ("cedar", "elm"),
        ]);
        let query = GraphQuery::new(&graph);
        let paths = query.reachable("amber", Direction::Dependencies).unwrap();
        let elm = paths
            .iter()
            .find(|path| path.nodes.last().map(String::as_str) == Some("elm"))
            .unwrap();

        assert_eq!(elm.nodes, ["amber", "birch", "elm"]);
    }

    #[test]
    fn an_unknown_node_is_not_reported_as_an_empty_dependency_set() {
        let graph = graph(&[("amber", "birch")]);
        let query = GraphQuery::new(&graph);

        assert_eq!(query.direct("missing", Direction::Dependencies), None);
        assert_eq!(query.reachable("missing", Direction::Dependencies), None);
        assert_eq!(
            query.direct("birch", Direction::Dependencies),
            Some(Vec::new())
        );
    }

    #[test]
    fn query_report_names_snapshot_policy_and_uncertainty() {
        let graph = graph(&[("amber", "birch")]);
        let report = GraphQuery::new(&graph)
            .report("amber", Direction::Dependencies, false)
            .unwrap();

        assert_eq!(report.snapshot_id, graph.build.id);
        assert_eq!(report.evidence_policy.name, "all_stored_edges");
        assert_eq!(report.uncertainty.resolved, 1);
        assert_eq!(report.paths.len(), 1);
    }
}
