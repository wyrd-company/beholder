//! The reference graph.
//!
//! Fan-in and fan-out are kept apart because they mean opposite things. Fan-in
//! is how much depends on a symbol, so it is the blast radius of changing it.
//! Fan-out is how much a symbol depends on, so it is how much has to be
//! understood to read it. Summing them would destroy both.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::phase1::FileAnalysis;
use crate::resolve::{Accuracy, Edge, ResolutionStats, ResolveInput, Resolver};

/// Degree of one symbol in the reference graph.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Degree {
    /// Distinct symbols that reference this one.
    pub fan_in: usize,
    /// Distinct symbols this one references.
    pub fan_out: usize,
}

/// The resolved reference graph over a file set.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    /// Stably sorted, deduplicated.
    pub edges: Vec<Edge>,
    /// Degree by symbol id. Only symbols with a non-zero degree appear.
    pub degree: BTreeMap<String, Degree>,
    /// Cohesion component index by symbol id, per file. See [`components`].
    pub component: BTreeMap<String, usize>,
    /// Number of connected components in each file, by repo-relative path.
    pub components_per_file: BTreeMap<String, usize>,
    /// What the edges are worth.
    pub accuracy: BTreeMap<String, Accuracy>,
    /// What resolution could and could not do.
    pub resolution: ResolutionStats,
}

/// Build the graph for a file set.
pub fn build(files: &[FileAnalysis], resolver: &dyn Resolver) -> Graph {
    let (edges, resolution) = resolver.resolve(&ResolveInput { files });

    let mut degree: BTreeMap<String, Degree> = BTreeMap::new();
    let mut out_neighbours: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut in_neighbours: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();

    for edge in &edges {
        out_neighbours
            .entry(edge.from.as_str())
            .or_default()
            .insert(edge.to.as_str());
        in_neighbours
            .entry(edge.to.as_str())
            .or_default()
            .insert(edge.from.as_str());
    }

    // Degree counts distinct partners, not occurrences: calling one function
    // twenty times is one dependency, not twenty.
    for (id, targets) in &out_neighbours {
        degree.entry((*id).to_owned()).or_default().fan_out = targets.len();
    }
    for (id, sources) in &in_neighbours {
        degree.entry((*id).to_owned()).or_default().fan_in = sources.len();
    }

    let mut component = BTreeMap::new();
    let mut components_per_file = BTreeMap::new();

    for file in files {
        let assignment = components(file, &edges);
        components_per_file.insert(file.path.clone(), count_components(&assignment));
        component.extend(assignment);
    }

    let mut accuracy = BTreeMap::new();
    for language in files.iter().filter_map(|f| f.language.as_deref()) {
        accuracy
            .entry(language.to_owned())
            .or_insert_with(|| resolver.accuracy(language));
    }

    Graph {
        edges,
        degree,
        component,
        components_per_file,
        accuracy,
        resolution,
    }
}

/// Partition one file's symbols into connected components.
///
/// Edges are read undirected and only within the file. One component at any
/// size is cohesive; three components are three files in a trenchcoat. This is
/// the honest form of "too many types in a file", because it measures whether
/// the contents refer to each other rather than how many of them there are.
///
/// Declaring a method on a type counts as connection alongside referring to it.
/// `Ledger` and `Ledger::tally` belong to the same component whether or not the
/// method body happens to name its own type, because a reader has to hold both
/// at once either way. This applies to cohesion only; it is not a reference and
/// never reaches fan-in or fan-out.
///
/// Component numbers are assigned in the file's symbol order, so they are stable
/// for a given file and mean nothing across files.
pub fn components(file: &FileAnalysis, edges: &[Edge]) -> BTreeMap<String, usize> {
    let ids: Vec<&str> = file.symbols.iter().map(|s| s.id.as_str()).collect();
    let mut parent: Vec<usize> = (0..ids.len()).collect();

    let position = |id: &str| ids.iter().position(|candidate| *candidate == id);

    for edge in edges {
        if let (Some(from), Some(to)) = (position(&edge.from), position(&edge.to)) {
            union(&mut parent, from, to);
        }
    }

    let separator = crate::resolve::separator_for(file);

    for (index, symbol) in file.symbols.iter().enumerate() {
        for (other, container) in file.symbols.iter().enumerate() {
            let name = crate::resolve::last_segment(&container.qualified_path, separator);
            if index != other
                && crate::resolve::scope_mentions(&symbol.qualified_path, separator, name)
            {
                union(&mut parent, index, other);
            }
        }
    }

    let mut labels: BTreeMap<usize, usize> = BTreeMap::new();
    let mut assignment = BTreeMap::new();

    for (index, id) in ids.iter().enumerate() {
        let root = find(&mut parent, index);
        let next = labels.len();
        let label = *labels.entry(root).or_insert(next);
        assignment.insert((*id).to_owned(), label);
    }

    assignment
}

fn count_components(assignment: &BTreeMap<String, usize>) -> usize {
    assignment.values().collect::<BTreeSet<_>>().len()
}

fn find(parent: &mut [usize], mut node: usize) -> usize {
    while parent[node] != node {
        parent[node] = parent[parent[node]];
        node = parent[node];
    }
    node
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let (a, b) = (find(parent, a), find(parent, b));
    if a != b {
        parent[b] = a;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase1;
    use crate::resolve::Heuristic;

    fn graph(files: &[(&str, &str)]) -> (Vec<FileAnalysis>, Graph) {
        let analyzed: Vec<_> = files
            .iter()
            .map(|(path, contents)| phase1::analyze(path, contents))
            .collect();
        let built = build(&analyzed, &Heuristic);
        (analyzed, built)
    }

    #[test]
    fn fan_in_and_fan_out_are_separate_values() {
        let (_, graph) = graph(&[(
            "src/lib.rs",
            "fn a() {\n    shared();\n}\nfn b() {\n    shared();\n}\nfn shared() {}\n",
        )]);

        let shared = &graph.degree["rust:src/lib.rs:function:shared"];
        assert_eq!(shared.fan_in, 2);
        assert_eq!(shared.fan_out, 0);

        let caller = &graph.degree["rust:src/lib.rs:function:a"];
        assert_eq!(caller.fan_in, 0);
        assert_eq!(caller.fan_out, 1);
    }

    #[test]
    fn degree_counts_distinct_partners_not_occurrences() {
        let (_, graph) = graph(&[(
            "src/lib.rs",
            "fn caller() {\n    once();\n    once();\n    once();\n}\nfn once() {}\n",
        )]);

        assert_eq!(graph.degree["rust:src/lib.rs:function:caller"].fan_out, 1);
        assert_eq!(graph.degree["rust:src/lib.rs:function:once"].fan_in, 1);
    }

    #[test]
    fn a_cohesive_file_is_one_component() {
        let (_, graph) = graph(&[(
            "src/lib.rs",
            "struct Ledger;\nimpl Ledger {\n    fn tally(&self) {\n        self.helper();\n    }\n    fn helper(&self) {}\n}\n",
        )]);

        assert_eq!(graph.components_per_file["src/lib.rs"], 1);
    }

    #[test]
    fn a_type_and_its_methods_are_one_component() {
        let (_, graph) = graph(&[(
            "src/lib.rs",
            "struct Ledger;\nimpl Ledger {\n    fn tally(&self) {}\n}\ntrait Paint { fn draw(&self); }\nimpl Paint for Ledger {\n    fn draw(&self) {}\n}\n",
        )]);

        // Ledger, Ledger::tally, Paint, Paint::draw and <Ledger as Paint>::draw
        // are all one cluster: the trait impl mentions both names.
        assert_eq!(graph.components_per_file["src/lib.rs"], 1);
    }

    #[test]
    fn unrelated_neighbours_are_separate_components() {
        // Two groups sharing a file and nothing else.
        let (_, graph) = graph(&[(
            "src/lib.rs",
            "fn a1() {\n    a2();\n}\nfn a2() {}\nfn b1() {\n    b2();\n}\nfn b2() {}\n",
        )]);

        assert_eq!(graph.components_per_file["src/lib.rs"], 2);

        let component = &graph.component;
        assert_eq!(
            component["rust:src/lib.rs:function:a1"],
            component["rust:src/lib.rs:function:a2"]
        );
        assert_ne!(
            component["rust:src/lib.rs:function:a1"],
            component["rust:src/lib.rs:function:b1"]
        );
    }

    #[test]
    fn cross_file_edges_do_not_join_components() {
        // Cohesion is a property of one file. A call out to another file says
        // nothing about whether this file hangs together.
        let (_, graph) = graph(&[
            ("src/helper.rs", "pub fn assist() {}\n"),
            (
                "src/lib.rs",
                "use crate::helper::assist;\nfn a() {\n    assist();\n}\nfn b() {\n    assist();\n}\n",
            ),
        ]);

        assert_eq!(graph.components_per_file["src/lib.rs"], 2);
    }

    #[test]
    fn the_graph_carries_an_error_bar() {
        let (_, graph) = graph(&[("src/lib.rs", "fn a() {}\n")]);

        let rust = &graph.accuracy["rust"];
        assert_eq!(rust.resolver, crate::resolve::HEURISTIC);
    }

    #[test]
    fn output_is_deterministic() {
        let files = [
            ("src/z.rs", "pub fn zeta() {}\n"),
            (
                "src/a.rs",
                "use crate::z::zeta;\nfn alpha() {\n    zeta();\n}\n",
            ),
        ];

        let (_, first) = graph(&files);
        let (_, second) = graph(&files);
        assert_eq!(first, second);
    }
}
