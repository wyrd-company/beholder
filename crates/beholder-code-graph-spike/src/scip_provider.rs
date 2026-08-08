// ---
// relationships:
//   implements: compiler-backed-code-graph
//   references: scip-tree-sitter-spike
// ---

//! SCIP adapter. Protocol types stop at this module's private boundary.

use std::collections::{BTreeMap, BTreeSet};

use protobuf::{Enum, Message};
use scip::types::{
    Document, Index, Occurrence, PositionEncoding as ScipPositionEncoding, Severity,
    SymbolInformation, SymbolRole,
};
use sha2::{Digest, Sha256};

use crate::model::{
    capsule_id, AnalyzedScope, BuildIdentity, CodeGraph, Diagnostic, DiagnosticSeverity, Edge,
    EdgeKind, Knowledge, Location, Node, NodeKind, NodeScope, PositionEncoding, Producer,
    Provenance, ReferenceFact, Resolution, SourceRange, UnknownReason, UnknownReasonKind,
    Validation, ValidationStatus, SCHEMA_VERSION,
};
use crate::provider::Provider;

const PROVIDER: &str = "scip";
const SCIP_TRANSPORT_VERSION: &str = "0.9.0";

#[derive(Clone, Debug)]
pub struct ScipProvider {
    pub build: BuildIdentity,
    pub requested_roots: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug)]
pub enum ScipProviderError {
    Decode(protobuf::Error),
    InputDigestMismatch { expected: String, actual: String },
    InvalidPath(String),
    MalformedRange(String),
    DefinitionCycle(String),
    Integrity(crate::model::IntegrityError),
}

impl std::fmt::Display for ScipProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "failed to decode SCIP: {error}"),
            Self::InputDigestMismatch { expected, actual } => write!(
                formatter,
                "SCIP input digest mismatch: expected {expected}, got {actual}"
            ),
            Self::InvalidPath(path) => write!(formatter, "SCIP document path is unsafe: {path}"),
            Self::MalformedRange(path) => write!(formatter, "malformed SCIP range in {path}"),
            Self::DefinitionCycle(symbol) => {
                write!(formatter, "SCIP definition relationship cycle at {symbol}")
            }
            Self::Integrity(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ScipProviderError {}

impl Provider for ScipProvider {
    type Error = ScipProviderError;

    fn produce(&self, input: &[u8]) -> Result<CodeGraph, Self::Error> {
        let actual = digest(input);
        if actual != self.build.provider_input_digest {
            return Err(ScipProviderError::InputDigestMismatch {
                expected: self.build.provider_input_digest.clone(),
                actual,
            });
        }
        let index = Index::parse_from_bytes(input).map_err(ScipProviderError::Decode)?;
        import(index, self)
    }
}

pub fn input_digest(input: &[u8]) -> String {
    digest(input)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SymbolKey {
    Global(String),
    Local { path: String, symbol: String },
}

impl SymbolKey {
    fn new(path: &str, symbol: &str) -> Self {
        if symbol.starts_with("local ") {
            Self::Local {
                path: path.to_owned(),
                symbol: symbol.to_owned(),
            }
        } else {
            Self::Global(symbol.to_owned())
        }
    }

    fn display(&self) -> &str {
        match self {
            Self::Global(symbol) | Self::Local { symbol, .. } => symbol,
        }
    }

    fn semantic_key(&self) -> String {
        match self {
            Self::Global(symbol) => symbol.clone(),
            Self::Local { path, symbol } => format!("{path}\0{symbol}"),
        }
    }

    fn id(&self, build_id: &str) -> String {
        match self {
            Self::Global(symbol) => format!(
                "node:{}",
                digest(format!("{build_id}\0global\0{symbol}").as_bytes())
            ),
            Self::Local { path, symbol } => format!(
                "node:{}",
                digest(format!("{build_id}\0local\0{path}\0{symbol}").as_bytes())
            ),
        }
    }
}

#[derive(Clone)]
struct Definition {
    key: SymbolKey,
    location: Location,
    body: Option<SourceRange>,
}

fn import(index: Index, provider: &ScipProvider) -> Result<CodeGraph, ScipProviderError> {
    let producer = producer(&index);
    let mut scope = AnalyzedScope {
        requested_roots: provider.requested_roots.clone(),
        limitations: provider.limitations.clone(),
        ..AnalyzedScope::default()
    };
    let mut definitions = BTreeMap::<SymbolKey, Vec<Definition>>::new();
    let mut information = BTreeMap::<SymbolKey, SymbolInformation>::new();
    let mut relationships = BTreeMap::<SymbolKey, BTreeSet<SymbolKey>>::new();
    let mut diagnostics = Vec::new();

    for document in &index.documents {
        if document.relative_path.is_empty() {
            return Err(ScipProviderError::InvalidPath(String::new()));
        }
        if !is_safe_path(&document.relative_path) {
            scope.external_documents.push(format!(
                "external:{}",
                digest(document.relative_path.as_bytes())
            ));
            continue;
        }
        scope
            .languages
            .insert(normalize_language(&document.language));
        scope
            .included_documents
            .push(document.relative_path.clone());
        for symbol in &document.symbols {
            let key = SymbolKey::new(&document.relative_path, &symbol.symbol);
            information
                .entry(key.clone())
                .or_insert_with(|| symbol.clone());
            collect_definition_relationships(
                &mut relationships,
                &key,
                &document.relative_path,
                symbol,
            );
        }
        for occurrence in &document.occurrences {
            for diagnostic in &occurrence.diagnostics {
                diagnostics.push(Diagnostic {
                    severity: match diagnostic.severity.enum_value().ok() {
                        Some(Severity::Error) => DiagnosticSeverity::Error,
                        Some(Severity::Warning) => DiagnosticSeverity::Warning,
                        _ => DiagnosticSeverity::Information,
                    },
                    code: diagnostic.code.clone(),
                    message: diagnostic.message.clone(),
                    location: occurrence_range(occurrence).map(|range| Location {
                        path: document.relative_path.clone(),
                        range,
                        encoding: position_encoding(document),
                    }),
                    provenance: provenance("diagnostic"),
                });
            }
            if occurrence.symbol.is_empty()
                || occurrence.symbol_roles & SymbolRole::Definition.value() == 0
            {
                continue;
            }
            let range = occurrence_range(occurrence)
                .ok_or_else(|| ScipProviderError::MalformedRange(document.relative_path.clone()))?;
            let key = SymbolKey::new(&document.relative_path, &occurrence.symbol);
            definitions
                .entry(key.clone())
                .or_default()
                .push(Definition {
                    key,
                    location: Location {
                        path: document.relative_path.clone(),
                        range,
                        encoding: position_encoding(document),
                    },
                    body: enclosing_range(occurrence),
                });
        }
    }
    scope.included_documents.sort();
    scope.included_documents.dedup();
    scope.external_documents.sort();
    scope.external_documents.dedup();

    let mut build = provider.build.clone();
    build.id = capsule_id(&build, &producer, &scope.requested_roots);

    let mut external = BTreeSet::new();
    for symbol in &index.external_symbols {
        let key = SymbolKey::Global(symbol.symbol.clone());
        external.insert(key.clone());
        information
            .entry(key.clone())
            .or_insert_with(|| symbol.clone());
        collect_definition_relationships(&mut relationships, &key, "", symbol);
    }

    let all_keys = definitions
        .keys()
        .chain(external.iter())
        .chain(information.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let node_map = all_keys
        .iter()
        .map(|key| (key.clone(), key.id(&build.id)))
        .collect::<BTreeMap<_, _>>();
    let external_node_ids = external
        .iter()
        .filter(|key| !definitions.contains_key(*key))
        .filter_map(|key| node_map.get(key).cloned())
        .collect::<BTreeSet<_>>();
    let mut definitions_by_path = BTreeMap::<&str, Vec<&Definition>>::new();
    for definition in definitions.values().flatten() {
        definitions_by_path
            .entry(definition.location.path.as_str())
            .or_default()
            .push(definition);
    }
    let mut nodes = Vec::new();
    for key in all_keys {
        let id = node_map[&key].clone();
        let info = information.get(&key);
        let provider_kind = info
            .and_then(|item| item.kind.enum_value().ok())
            .map(|kind| format!("{kind:?}"))
            .unwrap_or_else(|| "UnspecifiedKind".to_owned());
        let mut locations = definitions
            .get(&key)
            .into_iter()
            .flatten()
            .map(|definition| definition.location.clone())
            .collect::<Vec<_>>();
        locations.sort();
        locations.dedup();
        nodes.push(Node {
            id,
            semantic_key: key.semantic_key(),
            language: language_for(&key, &index.documents),
            kind: broad_kind(&provider_kind),
            provider_kind,
            display_name: info
                .map(|item| item.display_name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| key.display().to_owned()),
            scope: if matches!(key, SymbolKey::Local { .. }) {
                NodeScope::Local
            } else if external.contains(&key) && !definitions.contains_key(&key) {
                NodeScope::External
            } else {
                NodeScope::Workspace
            },
            definitions: locations,
            enclosing: info
                .map(|item| item.enclosing_symbol.as_str())
                .filter(|symbol| !symbol.is_empty())
                .and_then(|symbol| {
                    node_map
                        .get(&SymbolKey::new(key_path(&key), symbol))
                        .cloned()
                }),
            provenance: provenance("symbol_information"),
        });
    }
    nodes.sort_by(|left, right| left.id.cmp(&right.id));

    let mut facts = Vec::new();
    let mut edge_evidence = BTreeMap::<(String, String, EdgeKind), BTreeSet<String>>::new();
    for (document_index, document) in index.documents.iter().enumerate() {
        if !is_safe_path(&document.relative_path) {
            continue;
        }
        let references = grouped_references(document)?;
        for (location, targets) in references {
            let source =
                containing_definition(&definitions_by_path, &location).map(|key| key.id(&build.id));
            let mut candidates = BTreeSet::new();
            let mut raw_targets = Vec::new();
            let mut resolution_error = None;
            let edge_kind = if targets.iter().all(|target| target.import) {
                EdgeKind::Import
            } else {
                EdgeKind::Reference
            };
            for target in targets {
                let key = SymbolKey::new(&document.relative_path, &target.symbol);
                raw_targets.push(target.symbol);
                match resolve_key(&key, &relationships, &node_map, &mut Vec::new()) {
                    Ok(resolved) => candidates.extend(resolved),
                    Err(error) => resolution_error = Some(error),
                }
            }
            raw_targets.sort();
            let raw_target = raw_targets.join(" | ");
            let fact_id = format!(
                "reference:{}",
                digest(
                    format!(
                        "{document_index}\0{}\0{:?}\0{raw_target}",
                        location.path, location.range
                    )
                    .as_bytes()
                )
            );
            let outcome = if let Some(error) = resolution_error {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Warning,
                    code: "definition_cycle".to_owned(),
                    message: error.to_string(),
                    location: Some(location.clone()),
                    provenance: provenance("adapter_diagnostic"),
                });
                Resolution::Unknown {
                    reason: UnknownReason {
                        kind: UnknownReasonKind::DefinitionCycle,
                        details: "definition relationship cycle".to_owned(),
                    },
                }
            } else if candidates.len() > 1 {
                Resolution::Ambiguous {
                    candidates: candidates.iter().cloned().collect(),
                }
            } else if let Some(target) = candidates.iter().next() {
                if external_node_ids.contains(target) {
                    if let Some(from) = &source {
                        edge_evidence
                            .entry((from.clone(), target.clone(), edge_kind.clone()))
                            .or_default()
                            .insert(fact_id.clone());
                    }
                    Resolution::External {
                        node: target.clone(),
                    }
                } else {
                    if let Some(from) = &source {
                        edge_evidence
                            .entry((from.clone(), target.clone(), edge_kind.clone()))
                            .or_default()
                            .insert(fact_id.clone());
                    }
                    Resolution::Resolved {
                        node: target.clone(),
                    }
                }
            } else if targets_are_external(&raw_targets, &document.relative_path, &external) {
                Resolution::Unknown {
                    reason: UnknownReason {
                        kind: UnknownReasonKind::ExternalWithoutInformation,
                        details: "external target has no symbol information".to_owned(),
                    },
                }
            } else {
                Resolution::Unknown {
                    reason: UnknownReason {
                        kind: UnknownReasonKind::ProviderOmission,
                        details: format!(
                            "SCIP index has no matching symbol information: {raw_target}"
                        ),
                    },
                }
            };
            facts.push(ReferenceFact {
                id: fact_id,
                location: Some(location),
                source,
                raw_target,
                kind: edge_kind,
                outcome,
                provenance: provenance("occurrence"),
            });
        }
    }

    for (document_index, document) in index.documents.iter().enumerate() {
        if !is_safe_path(&document.relative_path) {
            continue;
        }
        add_relationship_edges(
            document_index,
            &document.relative_path,
            &document.symbols,
            &node_map,
            &external_node_ids,
            &mut facts,
            &mut edge_evidence,
        );
    }
    add_relationship_edges(
        usize::MAX,
        "",
        &index.external_symbols,
        &node_map,
        &external_node_ids,
        &mut facts,
        &mut edge_evidence,
    );

    facts.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = edge_evidence
        .into_iter()
        .map(|((from, to, kind), evidence)| Edge {
            id: format!(
                "edge:{}",
                digest(format!("{from}\0{to}\0{kind:?}").as_bytes())
            ),
            from,
            to,
            kind,
            evidence: evidence.into_iter().collect(),
            provenance: provenance("derived_edge"),
        })
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| left.id.cmp(&right.id));

    let mut graph = CodeGraph {
        schema_version: SCHEMA_VERSION,
        build,
        producer,
        scope,
        nodes,
        edges,
        references: facts,
        diagnostics,
        validations: vec![Validation {
            name: "provider_input_digest".to_owned(),
            status: ValidationStatus::Passed,
            details: provider.build.provider_input_digest.clone(),
            provenance: provenance("adapter_validation"),
        }],
    };
    graph.validate().map_err(ScipProviderError::Integrity)?;
    graph.validations.push(Validation {
        name: "graph_integrity".to_owned(),
        status: ValidationStatus::Passed,
        details: "every edge-eligible fact has exactly one evidence-backed projection".to_owned(),
        provenance: provenance("adapter_validation"),
    });
    Ok(graph)
}

fn producer(index: &Index) -> Producer {
    let tool = index.metadata.tool_info.as_ref();
    Producer {
        name: tool
            .map(|item| item.name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| PROVIDER.to_owned()),
        version: tool
            .map(|item| item.version.clone())
            .filter(|version| !version.is_empty())
            .map(Knowledge::observed)
            .unwrap_or_else(Knowledge::unknown),
        invocation: tool
            .filter(|item| !item.arguments.is_empty())
            .map(|item| Knowledge::observed(item.arguments.clone()))
            .unwrap_or_else(Knowledge::unknown),
        input_format: format!(
            "scip@{SCIP_TRANSPORT_VERSION};beholder-adapter@{}",
            env!("CARGO_PKG_VERSION")
        ),
    }
}

fn is_safe_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    let has_windows_drive_root =
        bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/';
    !path.is_empty()
        && !path.starts_with('/')
        && !has_windows_drive_root
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|component| matches!(component, "" | "." | ".."))
}

fn collect_definition_relationships(
    output: &mut BTreeMap<SymbolKey, BTreeSet<SymbolKey>>,
    owner: &SymbolKey,
    path: &str,
    information: &SymbolInformation,
) {
    for relationship in &information.relationships {
        if relationship.is_definition && !relationship.symbol.is_empty() {
            output
                .entry(owner.clone())
                .or_default()
                .insert(SymbolKey::new(path, &relationship.symbol));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TargetUse {
    symbol: String,
    import: bool,
}

fn grouped_references(
    document: &Document,
) -> Result<BTreeMap<Location, BTreeSet<TargetUse>>, ScipProviderError> {
    let mut grouped = BTreeMap::new();
    for occurrence in &document.occurrences {
        if occurrence.symbol.is_empty()
            || occurrence.symbol_roles & SymbolRole::Definition.value() != 0
        {
            continue;
        }
        let range = occurrence_range(occurrence)
            .ok_or_else(|| ScipProviderError::MalformedRange(document.relative_path.clone()))?;
        grouped
            .entry(Location {
                path: document.relative_path.clone(),
                range,
                encoding: position_encoding(document),
            })
            .or_insert_with(BTreeSet::new)
            .insert(TargetUse {
                symbol: occurrence.symbol.clone(),
                import: occurrence.symbol_roles & SymbolRole::Import.value() != 0,
            });
    }
    Ok(grouped)
}

fn containing_definition<'a>(
    definitions: &'a BTreeMap<&str, Vec<&'a Definition>>,
    location: &Location,
) -> Option<&'a SymbolKey> {
    definitions
        .get(location.path.as_str())
        .into_iter()
        .flatten()
        .filter(|definition| {
            definition
                .body
                .is_some_and(|body| body.contains(&location.range))
        })
        .min_by_key(|definition| definition.body.map(|body| body.size_key()))
        .map(|definition| &definition.key)
}

fn resolve_key(
    key: &SymbolKey,
    relationships: &BTreeMap<SymbolKey, BTreeSet<SymbolKey>>,
    nodes: &BTreeMap<SymbolKey, String>,
    visiting: &mut Vec<SymbolKey>,
) -> Result<BTreeSet<String>, ScipProviderError> {
    if visiting.contains(key) {
        return Err(ScipProviderError::DefinitionCycle(key.display().to_owned()));
    }
    if let Some(targets) = relationships.get(key) {
        visiting.push(key.clone());
        let mut resolved = BTreeSet::new();
        for target in targets {
            resolved.extend(resolve_key(target, relationships, nodes, visiting)?);
        }
        visiting.pop();
        Ok(resolved)
    } else {
        Ok(nodes.get(key).cloned().into_iter().collect())
    }
}

fn add_relationship_edges(
    document_index: usize,
    path: &str,
    symbols: &[SymbolInformation],
    nodes: &BTreeMap<SymbolKey, String>,
    external_nodes: &BTreeSet<String>,
    facts: &mut Vec<ReferenceFact>,
    edges: &mut BTreeMap<(String, String, EdgeKind), BTreeSet<String>>,
) {
    for (information_index, information) in symbols.iter().enumerate() {
        let owner = SymbolKey::new(path, &information.symbol);
        let Some(from) = nodes.get(&owner) else {
            continue;
        };
        for (relationship_index, relationship) in information.relationships.iter().enumerate() {
            let target = SymbolKey::new(path, &relationship.symbol);
            let Some(to) = nodes.get(&target) else {
                continue;
            };
            for kind in relationship_kinds(relationship) {
                let fact_id = format!(
                    "relationship:{}",
                    digest(
                        format!(
                            "{document_index}\0{}\0{}\0{}\0{information_index}\0{relationship_index}\0{kind:?}",
                            information.symbol, relationship.symbol, path,
                        )
                        .as_bytes()
                    )
                );
                let outcome = if external_nodes.contains(to) {
                    Resolution::External { node: to.clone() }
                } else {
                    Resolution::Resolved { node: to.clone() }
                };
                facts.push(ReferenceFact {
                    id: fact_id.clone(),
                    location: None,
                    source: Some(from.clone()),
                    raw_target: relationship.symbol.clone(),
                    kind: kind.clone(),
                    outcome,
                    provenance: provenance("symbol_relationship"),
                });
                edges
                    .entry((from.clone(), to.clone(), kind))
                    .or_default()
                    .insert(fact_id);
            }
        }
    }
}

fn relationship_kinds(relationship: &scip::types::Relationship) -> Vec<EdgeKind> {
    let mut kinds = Vec::new();
    if relationship.is_reference {
        kinds.push(EdgeKind::Reference);
    }
    if relationship.is_implementation {
        kinds.push(EdgeKind::Implementation);
    }
    if relationship.is_type_definition {
        kinds.push(EdgeKind::TypeDefinition);
    }
    if relationship.is_definition {
        kinds.push(EdgeKind::DefinitionAlias);
    }
    kinds
}

fn targets_are_external(targets: &[String], path: &str, external: &BTreeSet<SymbolKey>) -> bool {
    targets
        .iter()
        .all(|target| external.contains(&SymbolKey::new(path, target)))
}

fn position_encoding(document: &Document) -> PositionEncoding {
    match document.position_encoding.enum_value().ok() {
        Some(ScipPositionEncoding::UTF8CodeUnitOffsetFromLineStart) => {
            PositionEncoding::Utf8CodeUnit
        }
        Some(ScipPositionEncoding::UTF16CodeUnitOffsetFromLineStart) => {
            PositionEncoding::Utf16CodeUnit
        }
        Some(ScipPositionEncoding::UTF32CodeUnitOffsetFromLineStart) => {
            PositionEncoding::Utf32CodeUnit
        }
        _ => PositionEncoding::Unknown,
    }
}

fn occurrence_range(occurrence: &Occurrence) -> Option<SourceRange> {
    if occurrence.has_single_line_range() {
        let range = occurrence.single_line_range();
        return checked_range(
            range.line,
            range.start_character,
            range.line,
            range.end_character,
        );
    }
    if occurrence.has_multi_line_range() {
        let range = occurrence.multi_line_range();
        return checked_range(
            range.start_line,
            range.start_character,
            range.end_line,
            range.end_character,
        );
    }
    legacy_range(&occurrence.range)
}

fn enclosing_range(occurrence: &Occurrence) -> Option<SourceRange> {
    if occurrence.has_single_line_enclosing_range() {
        let range = occurrence.single_line_enclosing_range();
        return checked_range(
            range.line,
            range.start_character,
            range.line,
            range.end_character,
        );
    }
    if occurrence.has_multi_line_enclosing_range() {
        let range = occurrence.multi_line_enclosing_range();
        return checked_range(
            range.start_line,
            range.start_character,
            range.end_line,
            range.end_character,
        );
    }
    legacy_range(&occurrence.enclosing_range)
}

fn legacy_range(range: &[i32]) -> Option<SourceRange> {
    match range {
        [line, start, end] => checked_range(*line, *start, *line, *end),
        [start_line, start, end_line, end] => checked_range(*start_line, *start, *end_line, *end),
        _ => None,
    }
}

fn checked_range(
    start_line: i32,
    start_column: i32,
    end_line: i32,
    end_column: i32,
) -> Option<SourceRange> {
    let range = SourceRange {
        start_line: start_line.try_into().ok()?,
        start_column: start_column.try_into().ok()?,
        end_line: end_line.try_into().ok()?,
        end_column: end_column.try_into().ok()?,
    };
    ((range.start_line, range.start_column) <= (range.end_line, range.end_column)).then_some(range)
}

fn broad_kind(provider_kind: &str) -> NodeKind {
    match provider_kind {
        "Module" => NodeKind::Module,
        "Namespace" => NodeKind::Namespace,
        "Package" => NodeKind::Package,
        "Class" | "Struct" | "Trait" | "Interface" | "Enum" | "Union" | "Type" | "TypeAlias"
        | "AssociatedType" => NodeKind::Type,
        "Function" | "Method" | "StaticMethod" | "Constructor" | "TraitMethod" => {
            NodeKind::Callable
        }
        "Field" | "EnumMember" | "Property" | "Event" | "Constant" | "StaticVariable" => {
            NodeKind::Member
        }
        "Variable" => NodeKind::Variable,
        "Parameter" | "SelfParameter" | "TypeParameter" => NodeKind::Parameter,
        "Macro" => NodeKind::Macro,
        _ => NodeKind::Other,
    }
}

fn language_for(key: &SymbolKey, documents: &[Document]) -> String {
    match key {
        SymbolKey::Local { path, .. } => documents
            .iter()
            .find(|document| document.relative_path == *path)
            .map(|document| normalize_language(&document.language))
            .unwrap_or_else(|| "unknown".to_owned()),
        SymbolKey::Global(_) => documents
            .iter()
            .find(|document| {
                document
                    .symbols
                    .iter()
                    .any(|information| information.symbol == key.display())
            })
            .map(|document| normalize_language(&document.language))
            .unwrap_or_else(|| "unknown".to_owned()),
    }
}

fn normalize_language(language: &str) -> String {
    if language.is_empty() {
        "unknown".to_owned()
    } else {
        language.to_ascii_lowercase()
    }
}

fn key_path(key: &SymbolKey) -> &str {
    match key {
        SymbolKey::Local { path, .. } => path,
        SymbolKey::Global(_) => "",
    }
}

fn provenance(record_kind: &str) -> Provenance {
    Provenance {
        provider: PROVIDER.to_owned(),
        record_kind: record_kind.to_owned(),
    }
}

fn digest(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

#[cfg(test)]
mod tests {
    use protobuf::{Enum, Message, MessageField};
    use scip::types::symbol_information::Kind;
    use scip::types::{
        Document, Index, Metadata, Occurrence, Relationship, SymbolInformation, SymbolRole,
        ToolInfo,
    };

    use super::*;
    use crate::model::{CompilationUnit, EdgeKind, KnowledgeState, NodeScope, Resolution};

    #[test]
    fn provider_input_digest_is_load_bearing() {
        let bytes = index(vec![], vec![]).write_to_bytes().unwrap();
        let mut provider = provider(&bytes);
        provider.build.provider_input_digest = "previous-output".to_owned();

        assert!(matches!(
            provider.produce(&bytes),
            Err(ScipProviderError::InputDigestMismatch { .. })
        ));
    }

    #[test]
    fn local_symbols_are_document_scoped() {
        let first = document(
            "first.rs",
            vec![definition("local 0", [0, 3, 8], [0, 0, 1, 0])],
            vec![information("local 0", "first", Kind::Variable)],
        );
        let second = document(
            "second.rs",
            vec![definition("local 0", [0, 3, 8], [0, 0, 1, 0])],
            vec![information("local 0", "second", Kind::Variable)],
        );
        let bytes = index(vec![first, second], vec![]).write_to_bytes().unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();
        let locals = graph
            .nodes
            .iter()
            .filter(|node| node.scope == NodeScope::Local)
            .collect::<Vec<_>>();

        assert_eq!(locals.len(), 2);
        assert_ne!(locals[0].id, locals[1].id);
        assert_ne!(locals[0].semantic_key, locals[1].semantic_key);
        assert!(locals.iter().all(|node| node.scope == NodeScope::Local));
    }

    #[test]
    fn node_ids_are_snapshot_local() {
        let bytes = index(
            vec![document(
                "sample.rs",
                vec![definition("local 0", [0, 3, 8], [0, 0, 1, 0])],
                vec![information("local 0", "sample", Kind::Variable)],
            )],
            vec![],
        )
        .write_to_bytes()
        .unwrap();
        let first = provider(&bytes).produce(&bytes).unwrap();
        let mut second_provider = provider(&bytes);
        second_provider.build.repository = Knowledge::declared("another-repository".to_owned());
        let second = second_provider.produce(&bytes).unwrap();

        assert_ne!(first.build.id, second.build.id);
        assert_ne!(first.nodes[0].id, second.nodes[0].id);
    }

    #[test]
    fn capsule_identity_canonicalizes_requested_root_order() {
        let bytes = index(vec![], vec![]).write_to_bytes().unwrap();
        let mut first = provider(&bytes);
        first.requested_roots = vec!["src".to_owned(), "tests".to_owned()];
        let mut second = provider(&bytes);
        second.requested_roots = vec!["tests".to_owned(), "src".to_owned(), "src".to_owned()];

        assert_eq!(
            first.produce(&bytes).unwrap().build.id,
            second.produce(&bytes).unwrap().build.id
        );
    }

    #[test]
    fn provider_diagnostics_retain_location_and_provenance() {
        let mut occurrence = definition("local 0", [0, 3, 8], [0, 0, 1, 0]);
        occurrence.diagnostics.push(scip::types::Diagnostic {
            severity: Severity::Warning.into(),
            code: "sample-warning".to_owned(),
            message: "sample diagnostic".to_owned(),
            source: "sample-provider".to_owned(),
            ..scip::types::Diagnostic::default()
        });
        let bytes = index(
            vec![document(
                "sample.rs",
                vec![occurrence],
                vec![information("local 0", "sample", Kind::Variable)],
            )],
            vec![],
        )
        .write_to_bytes()
        .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();
        assert_eq!(graph.diagnostics[0].code, "sample-warning");
        assert_eq!(
            graph.diagnostics[0].location.as_ref().unwrap().path,
            "sample.rs"
        );
        assert_eq!(graph.diagnostics[0].provenance.provider, "scip");
    }

    #[test]
    fn import_role_creates_an_import_edge() {
        let occurrences = vec![
            definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 3, 0]),
            definition("sample pkg 1 birch().", [4, 3, 8], [4, 0, 5, 0]),
            reference(
                "sample pkg 1 birch().",
                [1, 4, 9],
                SymbolRole::Import.value(),
            ),
        ];
        let symbols = vec![
            information("sample pkg 1 amber().", "amber", Kind::Function),
            information("sample pkg 1 birch().", "birch", Kind::Function),
        ];
        let bytes = index(vec![document("sample.rs", occurrences, symbols)], vec![])
            .write_to_bytes()
            .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].kind, EdgeKind::Import);
    }

    #[test]
    fn ambiguous_target_emits_no_edge() {
        let occurrences = vec![
            definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 3, 0]),
            definition("sample pkg 1 birch().", [4, 3, 8], [4, 0, 5, 0]),
            definition("sample pkg 1 cedar().", [6, 3, 8], [6, 0, 7, 0]),
            reference("sample pkg 1 birch().", [1, 4, 9], 0),
            reference("sample pkg 1 cedar().", [1, 4, 9], 0),
        ];
        let symbols = vec![
            information("sample pkg 1 amber().", "amber", Kind::Function),
            information("sample pkg 1 birch().", "birch", Kind::Function),
            information("sample pkg 1 cedar().", "cedar", Kind::Function),
        ];
        let bytes = index(vec![document("sample.rs", occurrences, symbols)], vec![])
            .write_to_bytes()
            .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();

        assert!(graph.edges.is_empty());
        assert!(matches!(
            graph.references[0].outcome,
            Resolution::Ambiguous { .. }
        ));
    }

    #[test]
    fn definition_relationships_retarget_aliases_and_emit_typed_edges() {
        let target = "sample pkg 1 birch().";
        let alias = "sample pkg 1 alias().";
        let mut alias_information = information(alias, "alias", Kind::Function);
        alias_information.relationships.push(Relationship {
            symbol: target.to_owned(),
            is_definition: true,
            is_reference: true,
            ..Relationship::default()
        });
        let occurrences = vec![
            definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 3, 0]),
            definition(alias, [4, 3, 8], [4, 0, 5, 0]),
            definition(target, [6, 3, 8], [6, 0, 7, 0]),
            reference(alias, [1, 4, 9], 0),
        ];
        let symbols = vec![
            information("sample pkg 1 amber().", "amber", Kind::Function),
            alias_information,
            information(target, "birch", Kind::Function),
        ];
        let bytes = index(vec![document("sample.rs", occurrences, symbols)], vec![])
            .write_to_bytes()
            .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();

        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::DefinitionAlias));
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::Reference));
        let occurrence = graph
            .references
            .iter()
            .find(|fact| fact.location.is_some())
            .unwrap();
        let Resolution::Resolved { node } = &occurrence.outcome else {
            panic!("alias occurrence was not resolved");
        };
        assert_eq!(
            graph
                .nodes
                .iter()
                .find(|candidate| &candidate.id == node)
                .unwrap()
                .display_name,
            "birch"
        );
    }

    #[test]
    fn duplicate_occurrences_retain_every_evidence_record() {
        let occurrences = vec![
            definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 4, 0]),
            definition("sample pkg 1 birch().", [5, 3, 8], [5, 0, 6, 0]),
            reference("sample pkg 1 birch().", [1, 4, 9], 0),
            reference("sample pkg 1 birch().", [2, 4, 9], 0),
        ];
        let symbols = vec![
            information("sample pkg 1 amber().", "amber", Kind::Function),
            information("sample pkg 1 birch().", "birch", Kind::Function),
        ];
        let bytes = index(vec![document("sample.rs", occurrences, symbols)], vec![])
            .write_to_bytes()
            .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();

        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].evidence.len(), 2);
    }

    #[test]
    fn duplicate_symbol_relationships_retain_distinct_evidence_ids() {
        let source = "sample pkg 1 amber().";
        let target = "sample pkg 1 birch().";
        let mut source_information = information(source, "amber", Kind::Function);
        source_information.relationships.push(Relationship {
            symbol: target.to_owned(),
            is_reference: true,
            ..Relationship::default()
        });
        let bytes = index(
            vec![document(
                "sample.rs",
                vec![
                    definition(source, [0, 3, 8], [0, 0, 1, 0]),
                    definition(target, [2, 3, 8], [2, 0, 3, 0]),
                ],
                vec![
                    source_information.clone(),
                    source_information,
                    information(target, "birch", Kind::Function),
                ],
            )],
            vec![],
        )
        .write_to_bytes()
        .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].evidence.len(), 2);
        assert_ne!(graph.edges[0].evidence[0], graph.edges[0].evidence[1]);
    }

    #[test]
    fn duplicate_document_records_retain_distinct_evidence_ids() {
        let source = "sample pkg 1 amber().";
        let target = "sample pkg 1 birch().";
        let document = document(
            "sample.rs",
            vec![
                definition(source, [0, 3, 8], [0, 0, 2, 0]),
                definition(target, [3, 3, 8], [3, 0, 4, 0]),
                reference(target, [1, 3, 8], 0),
            ],
            vec![
                information(source, "amber", Kind::Function),
                information(target, "birch", Kind::Function),
            ],
        );
        let bytes = index(vec![document.clone(), document], vec![])
            .write_to_bytes()
            .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].evidence.len(), 2);
        assert_ne!(graph.edges[0].evidence[0], graph.edges[0].evidence[1]);
    }

    #[test]
    fn external_and_unknown_outcomes_are_distinct() {
        let external_symbol = information("sample pkg 1 external().", "external", Kind::Function);
        let occurrences = vec![
            definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 4, 0]),
            reference("sample pkg 1 external().", [1, 4, 9], 0),
            reference("sample pkg 1 absent().", [2, 4, 9], 0),
        ];
        let symbols = vec![information(
            "sample pkg 1 amber().",
            "amber",
            Kind::Function,
        )];
        let bytes = index(
            vec![document("sample.rs", occurrences, symbols)],
            vec![external_symbol],
        )
        .write_to_bytes()
        .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();

        assert!(graph
            .references
            .iter()
            .any(|fact| matches!(fact.outcome, Resolution::External { .. })));
        assert!(graph.references.iter().any(|fact| matches!(
            fact.outcome,
            Resolution::Unknown { ref reason }
                if reason.kind == UnknownReasonKind::ProviderOmission
                    && reason.details.contains("absent")
        )));
        assert_eq!(graph.edges.len(), 1, "external facts remain graph edges");
    }

    #[test]
    fn source_locations_retain_document_position_encoding() {
        let mut source = document(
            "sample.rs",
            vec![
                definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 2, 0]),
                reference("sample pkg 1 amber().", [1, 4, 9], 0),
            ],
            vec![information(
                "sample pkg 1 amber().",
                "amber",
                Kind::Function,
            )],
        );
        source.position_encoding = ScipPositionEncoding::UTF16CodeUnitOffsetFromLineStart.into();
        let bytes = index(vec![source], vec![]).write_to_bytes().unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();

        assert!(graph.nodes.iter().all(|node| node
            .definitions
            .iter()
            .all(|location| location.encoding == PositionEncoding::Utf16CodeUnit)));
        assert!(graph
            .references
            .iter()
            .filter_map(|fact| fact.location.as_ref())
            .all(|location| location.encoding == PositionEncoding::Utf16CodeUnit));
    }

    #[test]
    fn definition_alias_to_external_symbol_stays_external() {
        let target = "sample pkg 1 external().";
        let alias = "sample pkg 1 alias().";
        let mut alias_information = information(alias, "alias", Kind::Function);
        alias_information.relationships.push(Relationship {
            symbol: target.to_owned(),
            is_definition: true,
            ..Relationship::default()
        });
        let occurrences = vec![
            definition("sample pkg 1 amber().", [0, 3, 8], [0, 0, 3, 0]),
            definition(alias, [4, 3, 8], [4, 0, 5, 0]),
            reference(alias, [1, 4, 9], 0),
        ];
        let symbols = vec![
            information("sample pkg 1 amber().", "amber", Kind::Function),
            alias_information,
        ];
        let bytes = index(
            vec![document("sample.rs", occurrences, symbols)],
            vec![information(target, "external", Kind::Function)],
        )
        .write_to_bytes()
        .unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();
        assert!(graph.references.iter().any(|fact| {
            fact.location.is_some() && matches!(fact.outcome, Resolution::External { .. })
        }));
    }

    #[test]
    fn document_path_outside_root_is_opaque_external_scope() {
        for unsafe_path in ["../outside.rs", "C:/Users/example/outside.rs"] {
            let bytes = index(
                vec![document(
                    unsafe_path,
                    vec![definition("local 0", [0, 0, 1], [0, 0, 0, 1])],
                    vec![],
                )],
                vec![],
            )
            .write_to_bytes()
            .unwrap();

            let graph = provider(&bytes).produce(&bytes).unwrap();
            assert!(graph.nodes.is_empty());
            assert_eq!(graph.scope.external_documents.len(), 1);
            assert!(graph.scope.external_documents[0].starts_with("external:"));
            assert!(!graph.scope.external_documents[0].contains("outside"));
        }
    }

    #[test]
    fn incomplete_capsule_fields_stay_explicitly_unknown() {
        let bytes = index(vec![], vec![]).write_to_bytes().unwrap();
        let graph = provider(&bytes).produce(&bytes).unwrap();
        assert_eq!(graph.build.revision.state, KnowledgeState::Unknown);
        assert_eq!(graph.producer.invocation.state, KnowledgeState::Unknown);
        assert_eq!(
            graph.build.units.value.as_ref().unwrap()[0].features.state,
            KnowledgeState::Unknown
        );
    }

    #[test]
    fn absent_provider_language_is_explicitly_unknown() {
        let mut source = document(
            "sample.py",
            vec![definition("local 0", [0, 0, 1], [0, 0, 0, 1])],
            vec![information("local 0", "sample", Kind::Variable)],
        );
        source.language.clear();
        let bytes = index(vec![source], vec![]).write_to_bytes().unwrap();

        let graph = provider(&bytes).produce(&bytes).unwrap();
        assert_eq!(
            graph.scope.languages,
            BTreeSet::from(["unknown".to_owned()])
        );
        assert_eq!(graph.nodes[0].language, "unknown");
    }

    fn provider(bytes: &[u8]) -> ScipProvider {
        ScipProvider {
            build: BuildIdentity {
                id: String::new(),
                repository: Knowledge::declared("repository-sample".to_owned()),
                revision: Knowledge::unknown(),
                dirty: Knowledge::unknown(),
                source_digest: Knowledge::unknown(),
                configuration_digest: Knowledge::unknown(),
                provider_input_digest: input_digest(bytes),
                units: Knowledge::declared(vec![CompilationUnit {
                    id: "unit-sample".to_owned(),
                    language: "rust".to_owned(),
                    name: "sample".to_owned(),
                    kind: "library".to_owned(),
                    source_roots: vec!["src".to_owned()],
                    target: Knowledge::unknown(),
                    features: Knowledge::unknown(),
                }]),
            },
            requested_roots: vec!["src".to_owned()],
            limitations: vec!["test input".to_owned()],
        }
    }

    fn index(documents: Vec<Document>, external_symbols: Vec<SymbolInformation>) -> Index {
        Index {
            metadata: MessageField::some(Metadata {
                tool_info: MessageField::some(ToolInfo {
                    name: "sample-provider".to_owned(),
                    version: "1".to_owned(),
                    ..ToolInfo::default()
                }),
                ..Metadata::default()
            }),
            documents,
            external_symbols,
            ..Index::default()
        }
    }

    fn document(
        path: &str,
        occurrences: Vec<Occurrence>,
        symbols: Vec<SymbolInformation>,
    ) -> Document {
        Document {
            language: "rust".to_owned(),
            relative_path: path.to_owned(),
            occurrences,
            symbols,
            ..Document::default()
        }
    }

    fn information(symbol: &str, display_name: &str, kind: Kind) -> SymbolInformation {
        SymbolInformation {
            symbol: symbol.to_owned(),
            display_name: display_name.to_owned(),
            kind: kind.into(),
            ..SymbolInformation::default()
        }
    }

    fn definition(symbol: &str, range: [i32; 3], body: [i32; 4]) -> Occurrence {
        Occurrence {
            symbol: symbol.to_owned(),
            symbol_roles: SymbolRole::Definition.value(),
            range: range.to_vec(),
            enclosing_range: body.to_vec(),
            ..Occurrence::default()
        }
    }

    fn reference(symbol: &str, range: [i32; 3], roles: i32) -> Occurrence {
        Occurrence {
            symbol: symbol.to_owned(),
            symbol_roles: roles,
            range: range.to_vec(),
            ..Occurrence::default()
        }
    }
}
