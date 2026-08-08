// ---
// relationships:
//   implements: compiler-backed-code-graph
// ---

//! Experimental provider/consumer boundary for compiler-backed code graphs.

pub mod coupling;
pub mod model;
pub mod provider;
pub mod query;
pub mod scip_provider;

#[cfg(test)]
pub(crate) mod test_support {
    use std::collections::BTreeSet;

    use crate::model::{
        capsule_id, AnalyzedScope, BuildIdentity, CodeGraph, Edge, EdgeKind, Knowledge, Node,
        NodeKind, NodeScope, Producer, Provenance, ReferenceFact, Resolution, SCHEMA_VERSION,
    };

    pub fn graph(pairs: &[(&str, &str)]) -> CodeGraph {
        let names = pairs
            .iter()
            .flat_map(|(from, to)| [*from, *to])
            .collect::<BTreeSet<_>>();
        let nodes = names
            .into_iter()
            .map(|name| Node {
                id: name.to_owned(),
                semantic_key: name.to_owned(),
                language: "sample".to_owned(),
                kind: NodeKind::Callable,
                provider_kind: "Function".to_owned(),
                display_name: name.to_owned(),
                scope: NodeScope::Workspace,
                definitions: Vec::new(),
                enclosing: None,
                provenance: provenance(),
            })
            .collect();
        let mut references = Vec::new();
        let edges = pairs
            .iter()
            .enumerate()
            .map(|(index, (from, to))| {
                let evidence = format!("reference-{index}");
                references.push(ReferenceFact {
                    id: evidence.clone(),
                    location: None,
                    source: Some((*from).to_owned()),
                    raw_target: (*to).to_owned(),
                    kind: EdgeKind::Reference,
                    outcome: Resolution::Resolved {
                        node: (*to).to_owned(),
                    },
                    provenance: provenance(),
                });
                Edge {
                    id: format!("edge-{index}"),
                    from: (*from).to_owned(),
                    to: (*to).to_owned(),
                    kind: EdgeKind::Reference,
                    evidence: vec![evidence],
                    provenance: provenance(),
                }
            })
            .collect();
        let mut graph = CodeGraph {
            schema_version: SCHEMA_VERSION,
            build: BuildIdentity {
                id: String::new(),
                repository: Knowledge::unknown(),
                revision: Knowledge::unknown(),
                dirty: Knowledge::unknown(),
                source_digest: Knowledge::unknown(),
                configuration_digest: Knowledge::unknown(),
                provider_input_digest: "input".to_owned(),
                units: Knowledge::unknown(),
            },
            producer: Producer {
                name: "sample".to_owned(),
                version: Knowledge::unknown(),
                invocation: Knowledge::unknown(),
                input_format: "sample".to_owned(),
            },
            scope: AnalyzedScope::default(),
            nodes,
            edges,
            references,
            diagnostics: Vec::new(),
            validations: Vec::new(),
        };
        graph.build.id = capsule_id(&graph.build, &graph.producer, &graph.scope.requested_roots);
        graph
    }

    fn provenance() -> Provenance {
        Provenance {
            provider: "sample".to_owned(),
            record_kind: "sample".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{EdgeKind, Resolution};
    use crate::test_support::graph;

    #[test]
    fn graph_integrity_rejects_a_dangling_node() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.nodes.pop();
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::DanglingNode(_))
        ));
    }

    #[test]
    fn graph_integrity_rejects_dangling_evidence() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.references.clear();
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::DanglingEvidence(_))
        ));
    }

    #[test]
    fn graph_integrity_rejects_build_identity_tampering() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.build.id = "not-the-capsule-digest".to_owned();
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::BuildIdentityMismatch)
        ));
    }

    #[test]
    fn graph_integrity_rejects_a_dangling_resolution() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.references[0].outcome = Resolution::Resolved {
            node: "missing".to_owned(),
        };
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::DanglingNode(node)) if node == "missing"
        ));
    }

    #[test]
    fn graph_integrity_requires_edge_evidence() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.edges[0].evidence.clear();
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::MissingEvidence(_))
        ));
    }

    #[test]
    fn graph_integrity_rejects_empty_ambiguity() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.references[0].outcome = Resolution::Ambiguous {
            candidates: Vec::new(),
        };
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::EmptyCandidateSet(_))
        ));
    }

    #[test]
    fn graph_integrity_checks_external_outcome_against_node_scope() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.references[0].outcome = Resolution::External {
            node: "birch".to_owned(),
        };
        assert!(matches!(
            graph.validate(),
            Err(crate::model::IntegrityError::ResolutionScopeMismatch(_))
        ));
    }

    #[test]
    fn provider_specific_edge_kind_keeps_its_stable_name() {
        let kind = EdgeKind::ProviderSpecific {
            kind: "sample.stable-kind".to_owned(),
        };
        let json = serde_json::to_string(&kind).unwrap();
        let round_trip: EdgeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, kind);
        assert!(json.contains("sample.stable-kind"));
    }
}
