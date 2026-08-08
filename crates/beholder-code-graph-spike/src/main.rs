// ---
// relationships:
//   implements: compiler-backed-code-graph
// ---

//! Experimental file boundary between a SCIP provider and graph consumers.

use std::collections::BTreeMap;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

use anyhow::{Context, Result};
use beholder_code_graph_spike::coupling;
use beholder_code_graph_spike::model::{BuildIdentity, CodeGraph, Knowledge};
use beholder_code_graph_spike::provider::Provider;
use beholder_code_graph_spike::query::{Direction, GraphQuery};
use beholder_code_graph_spike::scip_provider::{input_digest, ScipProvider};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "beholder-code-graph-spike", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Import a provider index into the language-neutral graph snapshot.
    Import {
        #[arg(long)]
        index: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        repository: String,
        #[arg(long)]
        revision: Option<String>,
        #[arg(long)]
        source_digest: Option<String>,
        #[arg(long)]
        configuration_digest: Option<String>,
        #[arg(long = "root")]
        requested_roots: Vec<String>,
    },
    /// Return direct or transitive dependency paths from a stored snapshot.
    Query {
        graph: PathBuf,
        node: String,
        #[arg(long, value_enum, default_value_t = QueryDirection::Dependencies)]
        direction: QueryDirection,
        #[arg(long)]
        transitive: bool,
    },
    /// Reproduce coupling measurements from stored nodes and edges.
    Coupling { graph: PathBuf },
}

#[derive(Clone, Copy, ValueEnum)]
enum QueryDirection {
    Dependencies,
    Dependants,
}

impl From<QueryDirection> for Direction {
    fn from(value: QueryDirection) -> Self {
        match value {
            QueryDirection::Dependencies => Self::Dependencies,
            QueryDirection::Dependants => Self::Dependants,
        }
    }
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Import {
            index,
            output,
            repository,
            revision,
            source_digest,
            configuration_digest,
            requested_roots,
        } => {
            let bytes = fs::read(&index)
                .with_context(|| format!("reading provider index {}", index.display()))?;
            let provider = ScipProvider {
                build: BuildIdentity {
                    id: String::new(),
                    repository: Knowledge::declared(repository),
                    revision: revision
                        .map(Knowledge::declared)
                        .unwrap_or_else(Knowledge::unknown),
                    dirty: Knowledge::unknown(),
                    source_digest: source_digest
                        .map(Knowledge::declared)
                        .unwrap_or_else(Knowledge::unknown),
                    configuration_digest: configuration_digest
                        .map(Knowledge::declared)
                        .unwrap_or_else(Knowledge::unknown),
                    provider_input_digest: input_digest(&bytes),
                    units: Knowledge::unknown(),
                },
                requested_roots,
                limitations: vec![
                    "SCIP does not carry configured compilation units or coverage accounting"
                        .to_owned(),
                    "invocation wrapper did not supply document dispositions".to_owned(),
                ],
            };
            let graph = provider.produce(&bytes)?;
            let file = fs::File::create(&output)
                .with_context(|| format!("creating graph snapshot {}", output.display()))?;
            serde_json::to_writer_pretty(BufWriter::new(file), &graph)
                .with_context(|| format!("writing graph snapshot {}", output.display()))?;
            println!("{}", serde_json::to_string(&GraphSummary::from(&graph))?);
        }
        Command::Query {
            graph,
            node,
            direction,
            transitive,
        } => {
            let graph = read_graph(&graph)?;
            graph.validate()?;
            let query = GraphQuery::new(&graph);
            let report = query
                .report(&node, direction.into(), transitive)
                .with_context(|| format!("unknown graph node {node}"))?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::Coupling { graph } => {
            let graph = read_graph(&graph)?;
            graph.validate()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&coupling::analyze(&graph))?
            );
        }
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct GraphSummary {
    producer: String,
    languages: Vec<String>,
    documents: usize,
    external_documents: usize,
    nodes: usize,
    edges: usize,
    references: usize,
    diagnostics: usize,
    outcomes: BTreeMap<&'static str, usize>,
}

impl From<&CodeGraph> for GraphSummary {
    fn from(graph: &CodeGraph) -> Self {
        let mut outcomes = BTreeMap::new();
        for fact in &graph.references {
            let outcome = match fact.outcome {
                beholder_code_graph_spike::model::Resolution::Resolved { .. } => "resolved",
                beholder_code_graph_spike::model::Resolution::Ambiguous { .. } => "ambiguous",
                beholder_code_graph_spike::model::Resolution::External { .. } => "external",
                beholder_code_graph_spike::model::Resolution::Unknown { .. } => "unknown",
            };
            *outcomes.entry(outcome).or_insert(0) += 1;
        }
        Self {
            producer: graph.producer.name.clone(),
            languages: graph.scope.languages.iter().cloned().collect(),
            documents: graph.scope.included_documents.len(),
            external_documents: graph.scope.external_documents.len(),
            nodes: graph.nodes.len(),
            edges: graph.edges.len(),
            references: graph.references.len(),
            diagnostics: graph.diagnostics.len(),
            outcomes,
        }
    }
}

fn read_graph(path: &PathBuf) -> Result<CodeGraph> {
    let bytes = fs::read(path).with_context(|| format!("reading graph {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("decoding graph {}", path.display()))
}
