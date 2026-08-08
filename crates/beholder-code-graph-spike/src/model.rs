// ---
// relationships:
//   implements: compiler-backed-code-graph
// ---

//! Language-neutral records exchanged between semantic providers and consumers.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGraph {
    pub schema_version: u32,
    pub build: BuildIdentity,
    pub producer: Producer,
    pub scope: AnalyzedScope,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub references: Vec<ReferenceFact>,
    pub diagnostics: Vec<Diagnostic>,
    pub validations: Vec<Validation>,
}

impl CodeGraph {
    pub fn validate(&self) -> Result<(), IntegrityError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(IntegrityError::UnsupportedSchema(self.schema_version));
        }
        let nodes = self
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<BTreeSet<_>>();
        let node_scopes = self
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node.scope))
            .collect::<BTreeMap<_, _>>();
        if nodes.len() != self.nodes.len() {
            return Err(IntegrityError::DuplicateNode);
        }
        if self.build.id != capsule_id(&self.build, &self.producer, &self.scope.requested_roots) {
            return Err(IntegrityError::BuildIdentityMismatch);
        }
        validate_knowledge("build.repository", &self.build.repository)?;
        validate_knowledge("build.revision", &self.build.revision)?;
        validate_knowledge("build.dirty", &self.build.dirty)?;
        validate_knowledge("build.source_digest", &self.build.source_digest)?;
        validate_knowledge(
            "build.configuration_digest",
            &self.build.configuration_digest,
        )?;
        validate_knowledge("build.units", &self.build.units)?;
        validate_knowledge("producer.version", &self.producer.version)?;
        validate_knowledge("producer.invocation", &self.producer.invocation)?;
        if let Some(units) = &self.build.units.value {
            for unit in units {
                validate_knowledge("build.units[].target", &unit.target)?;
                validate_knowledge("build.units[].features", &unit.features)?;
            }
        }
        validate_scope(&self.scope)?;
        for node in &self.nodes {
            if node
                .enclosing
                .as_ref()
                .is_some_and(|enclosing| !nodes.contains(enclosing.as_str()))
            {
                return Err(IntegrityError::DanglingNode(
                    node.enclosing.clone().unwrap_or_default(),
                ));
            }
        }

        let mut evidence = BTreeSet::new();
        let mut evidence_facts = BTreeMap::new();
        for fact in &self.references {
            if !evidence.insert(fact.id.as_str()) {
                return Err(IntegrityError::DuplicateReference(fact.id.clone()));
            }
            evidence_facts.insert(fact.id.as_str(), fact);
        }
        for fact in &self.references {
            if fact
                .source
                .as_ref()
                .is_some_and(|source| !nodes.contains(source.as_str()))
            {
                return Err(IntegrityError::DanglingNode(
                    fact.source.clone().unwrap_or_default(),
                ));
            }
            let targets = match &fact.outcome {
                Resolution::Resolved { node } => {
                    if node_scopes.get(node.as_str()) == Some(&NodeScope::External) {
                        return Err(IntegrityError::ResolutionScopeMismatch(fact.id.clone()));
                    }
                    std::slice::from_ref(node)
                }
                Resolution::External { node } => {
                    if node_scopes.get(node.as_str()) != Some(&NodeScope::External) {
                        return Err(IntegrityError::ResolutionScopeMismatch(fact.id.clone()));
                    }
                    std::slice::from_ref(node)
                }
                Resolution::Ambiguous { candidates } => {
                    if candidates.is_empty() {
                        return Err(IntegrityError::EmptyCandidateSet(fact.id.clone()));
                    }
                    candidates.as_slice()
                }
                Resolution::Unknown { .. } => &[],
            };
            for target in targets {
                if !nodes.contains(target.as_str()) {
                    return Err(IntegrityError::DanglingNode(target.clone()));
                }
            }
        }

        let mut edge_ids = BTreeSet::new();
        for edge in &self.edges {
            if !edge_ids.insert(edge.id.as_str()) {
                return Err(IntegrityError::DuplicateEdge);
            }
            if !nodes.contains(edge.from.as_str()) {
                return Err(IntegrityError::DanglingNode(edge.from.clone()));
            }
            if !nodes.contains(edge.to.as_str()) {
                return Err(IntegrityError::DanglingNode(edge.to.clone()));
            }
            if edge.evidence.is_empty() {
                return Err(IntegrityError::MissingEvidence(edge.id.clone()));
            }
            for evidence_id in &edge.evidence {
                let Some(fact) = evidence_facts.get(evidence_id.as_str()) else {
                    return Err(IntegrityError::DanglingEvidence(evidence_id.clone()));
                };
                let target = match &fact.outcome {
                    Resolution::Resolved { node } | Resolution::External { node } => node,
                    Resolution::Ambiguous { .. } | Resolution::Unknown { .. } => {
                        return Err(IntegrityError::UnsupportedEvidence(evidence_id.clone()));
                    }
                };
                if fact.source.as_ref() != Some(&edge.from)
                    || target != &edge.to
                    || fact.kind != edge.kind
                {
                    return Err(IntegrityError::UnsupportedEvidence(evidence_id.clone()));
                }
            }
        }
        for node in &self.nodes {
            if let Some(enclosing) = &node.enclosing {
                if !nodes.contains(enclosing.as_str()) {
                    return Err(IntegrityError::DanglingNode(enclosing.clone()));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildIdentity {
    pub id: String,
    pub repository: Knowledge<String>,
    pub revision: Knowledge<String>,
    pub dirty: Knowledge<bool>,
    pub source_digest: Knowledge<String>,
    pub configuration_digest: Knowledge<String>,
    pub provider_input_digest: String,
    pub units: Knowledge<Vec<CompilationUnit>>,
}

/// Compute the snapshot identity from normalized, machine-independent capsule fields.
pub fn capsule_id(
    build: &BuildIdentity,
    producer: &Producer,
    requested_roots: &[String],
) -> String {
    let mut normalized = build.clone();
    normalized.id.clear();
    if let Some(units) = normalized.units.value.as_mut() {
        for unit in units.iter_mut() {
            unit.source_roots.sort();
            unit.source_roots.dedup();
            if let Some(features) = unit.features.value.as_mut() {
                features.sort();
                features.dedup();
            }
        }
        units.sort_by(|left, right| {
            (&left.id, &left.language, &left.name, &left.kind).cmp(&(
                &right.id,
                &right.language,
                &right.name,
                &right.kind,
            ))
        });
    }
    let mut roots = requested_roots.to_vec();
    roots.sort();
    roots.dedup();
    let bytes = serde_json::to_vec(&(normalized, producer, roots))
        .expect("capsule identity fields always serialize");
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationUnit {
    pub id: String,
    pub language: String,
    pub name: String,
    pub kind: String,
    pub source_roots: Vec<String>,
    pub target: Knowledge<String>,
    pub features: Knowledge<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Knowledge<T> {
    pub state: KnowledgeState,
    pub value: Option<T>,
}

impl<T> Knowledge<T> {
    pub fn observed(value: T) -> Self {
        Self {
            state: KnowledgeState::Observed,
            value: Some(value),
        }
    }

    pub fn declared(value: T) -> Self {
        Self {
            state: KnowledgeState::Declared,
            value: Some(value),
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: KnowledgeState::Unknown,
            value: None,
        }
    }

    fn is_valid(&self) -> bool {
        matches!(
            (&self.state, &self.value),
            (KnowledgeState::Observed | KnowledgeState::Declared, Some(_))
                | (KnowledgeState::Unknown, None)
        )
    }
}

fn validate_knowledge<T>(name: &str, knowledge: &Knowledge<T>) -> Result<(), IntegrityError> {
    knowledge
        .is_valid()
        .then_some(())
        .ok_or_else(|| IntegrityError::ContradictoryKnowledge(name.to_owned()))
}

fn validate_scope(scope: &AnalyzedScope) -> Result<(), IntegrityError> {
    let mut dispositions = BTreeMap::<String, &'static str>::new();
    let mut record = |path: &str, disposition| {
        if let Some(previous) = dispositions.insert(path.to_owned(), disposition) {
            return Err(IntegrityError::ContradictoryScope {
                path: path.to_owned(),
                first: previous,
                second: disposition,
            });
        }
        Ok(())
    };
    for path in &scope.included_documents {
        record(path, "included")?;
    }
    for path in scope.excluded_documents.keys() {
        record(path, "excluded")?;
    }
    for path in &scope.missing_documents {
        record(path, "missing")?;
    }
    for path in &scope.generated_documents {
        record(path, "generated")?;
    }
    for path in &scope.external_documents {
        record(path, "external")?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeState {
    Observed,
    Declared,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: Knowledge<String>,
    pub invocation: Knowledge<Vec<String>>,
    pub input_format: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalyzedScope {
    pub languages: BTreeSet<String>,
    pub requested_roots: Vec<String>,
    pub included_documents: Vec<String>,
    pub excluded_documents: BTreeMap<String, String>,
    pub missing_documents: Vec<String>,
    pub generated_documents: Vec<String>,
    pub external_documents: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub semantic_key: String,
    pub language: String,
    pub kind: NodeKind,
    pub provider_kind: String,
    pub display_name: String,
    pub scope: NodeScope,
    pub definitions: Vec<Location>,
    pub enclosing: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Module,
    Namespace,
    Package,
    Type,
    Callable,
    Member,
    Variable,
    Parameter,
    Macro,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeScope {
    Workspace,
    External,
    Local,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    pub evidence: Vec<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Reference,
    Import,
    Implementation,
    TypeDefinition,
    DefinitionAlias,
    ProviderSpecific { kind: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceFact {
    pub id: String,
    pub location: Option<Location>,
    pub source: Option<String>,
    pub raw_target: String,
    pub kind: EdgeKind,
    pub outcome: Resolution,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Resolution {
    Resolved { node: String },
    Ambiguous { candidates: Vec<String> },
    External { node: String },
    Unknown { reason: UnknownReason },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePolicy {
    pub name: String,
    pub providers: BTreeSet<String>,
    pub edge_kinds: BTreeSet<EdgeKind>,
}

impl EvidencePolicy {
    /// The prototype policy accepts every validated stored edge.
    pub fn all_stored(graph: &CodeGraph) -> Self {
        Self {
            name: "all_stored_edges".to_owned(),
            providers: graph
                .edges
                .iter()
                .map(|edge| edge.provenance.provider.clone())
                .collect(),
            edge_kinds: graph.edges.iter().map(|edge| edge.kind.clone()).collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncertaintyCounts {
    pub resolved: usize,
    pub ambiguous: usize,
    pub external: usize,
    pub unknown: usize,
}

impl From<&CodeGraph> for UncertaintyCounts {
    fn from(graph: &CodeGraph) -> Self {
        let mut counts = Self::default();
        for fact in &graph.references {
            match fact.outcome {
                Resolution::Resolved { .. } => counts.resolved += 1,
                Resolution::Ambiguous { .. } => counts.ambiguous += 1,
                Resolution::External { .. } => counts.external += 1,
                Resolution::Unknown { .. } => counts.unknown += 1,
            }
        }
        counts
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownReason {
    pub kind: UnknownReasonKind,
    pub details: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownReasonKind {
    AbsentFromScope,
    ExternalWithoutInformation,
    DefinitionCycle,
    ProviderOmission,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Location {
    pub path: String,
    pub range: SourceRange,
    pub encoding: PositionEncoding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionEncoding {
    Utf8CodeUnit,
    Utf16CodeUnit,
    Utf32CodeUnit,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceRange {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl SourceRange {
    pub fn contains(&self, other: &Self) -> bool {
        (self.start_line, self.start_column) <= (other.start_line, other.start_column)
            && (other.end_line, other.end_column) <= (self.end_line, self.end_column)
    }

    pub fn size_key(&self) -> (u32, u32) {
        (
            self.end_line.saturating_sub(self.start_line),
            self.end_column.saturating_sub(self.start_column),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub provider: String,
    pub record_kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub location: Option<Location>,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Information,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validation {
    pub name: String,
    pub status: ValidationStatus,
    pub details: String,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntegrityError {
    UnsupportedSchema(u32),
    BuildIdentityMismatch,
    DuplicateNode,
    DuplicateEdge,
    DuplicateReference(String),
    DanglingNode(String),
    DanglingEvidence(String),
    UnsupportedEvidence(String),
    MissingEvidence(String),
    EmptyCandidateSet(String),
    ResolutionScopeMismatch(String),
    ContradictoryKnowledge(String),
    ContradictoryScope {
        path: String,
        first: &'static str,
        second: &'static str,
    },
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchema(version) => write!(formatter, "unsupported schema {version}"),
            Self::BuildIdentityMismatch => formatter.write_str("build capsule identity mismatch"),
            Self::DuplicateNode => formatter.write_str("duplicate node id"),
            Self::DuplicateEdge => formatter.write_str("duplicate edge id"),
            Self::DuplicateReference(reference) => {
                write!(formatter, "duplicate reference id {reference}")
            }
            Self::DanglingNode(node) => write!(formatter, "edge names missing node {node}"),
            Self::DanglingEvidence(evidence) => {
                write!(formatter, "edge names missing evidence {evidence}")
            }
            Self::UnsupportedEvidence(evidence) => {
                write!(formatter, "fact does not support edge {evidence}")
            }
            Self::MissingEvidence(edge) => write!(formatter, "edge has no evidence {edge}"),
            Self::EmptyCandidateSet(reference) => {
                write!(
                    formatter,
                    "ambiguous reference has no candidates {reference}"
                )
            }
            Self::ResolutionScopeMismatch(reference) => {
                write!(
                    formatter,
                    "reference outcome contradicts node scope {reference}"
                )
            }
            Self::ContradictoryKnowledge(field) => {
                write!(formatter, "knowledge state contradicts value for {field}")
            }
            Self::ContradictoryScope {
                path,
                first,
                second,
            } => write!(
                formatter,
                "document {path} has conflicting scope dispositions {first} and {second}"
            ),
        }
    }
}

impl std::error::Error for IntegrityError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::graph;

    #[test]
    fn integrity_rejects_knowledge_state_value_contradictions() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.build.revision = Knowledge {
            state: KnowledgeState::Unknown,
            value: Some("revision".to_owned()),
        };
        graph.build.id = capsule_id(&graph.build, &graph.producer, &graph.scope.requested_roots);

        assert_eq!(
            graph.validate(),
            Err(IntegrityError::ContradictoryKnowledge(
                "build.revision".to_owned()
            ))
        );

        graph.build.revision = Knowledge {
            state: KnowledgeState::Observed,
            value: None,
        };
        graph.build.id = capsule_id(&graph.build, &graph.producer, &graph.scope.requested_roots);
        assert_eq!(
            graph.validate(),
            Err(IntegrityError::ContradictoryKnowledge(
                "build.revision".to_owned()
            ))
        );
    }

    #[test]
    fn integrity_rejects_conflicting_document_dispositions() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.scope.included_documents.push("sample.txt".to_owned());
        graph.scope.missing_documents.push("sample.txt".to_owned());

        assert!(matches!(
            graph.validate(),
            Err(IntegrityError::ContradictoryScope { ref path, .. }) if path == "sample.txt"
        ));
    }

    #[test]
    fn integrity_rejects_evidence_that_does_not_support_an_edge() {
        let mut graph = graph(&[("amber", "birch")]);
        graph.references[0].source = Some("birch".to_owned());

        assert_eq!(
            graph.validate(),
            Err(IntegrityError::UnsupportedEvidence(
                "reference-0".to_owned()
            ))
        );
    }
}
