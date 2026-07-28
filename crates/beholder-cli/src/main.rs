//! `beholder` — a structural index of a codebase.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use beholder::analysis::{self, Fingerprint};
use beholder::delta;
use beholder::store::{Store, DEFAULT_REF, REFSPEC};
use beholder::{jsonl, Config};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "beholder", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Analyze a working tree and write symbols.jsonl and files.jsonl.
    Index(IndexArgs),
    /// Compare two revisions and report changed symbols.
    Delta(DeltaArgs),
    /// Rank the symbols a change touched, most risky first.
    Report(ReportArgs),
    /// Report on the stored index ref.
    Store(StoreArgs),
    /// Run delta across a run of revisions and count phantom changes.
    ///
    /// A phantom is a symbol change reported in a file that did not change. It
    /// means symbol identity moved when the code did not.
    AuditIdentity(AuditArgs),
}

#[derive(Args)]
struct IndexArgs {
    /// Repository root. Defaults to the current directory.
    #[arg(default_value = ".")]
    root: PathBuf,
    /// Directory to write JSONL into. Defaults to the repository root.
    #[arg(long)]
    out_dir: Option<PathBuf>,
    /// Write results to the beholder ref.
    #[arg(long)]
    store: bool,
    /// Reuse stored phase 1 results where they are valid.
    #[arg(long)]
    use_store: bool,
}

#[derive(Args)]
struct DeltaArgs {
    /// Revision to compare from.
    before: String,
    /// Revision to compare to.
    after: String,
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Read and append phase results in the beholder ref.
    #[arg(long)]
    use_store: bool,
}

#[derive(Args)]
struct ReportArgs {
    before: String,
    after: String,
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Which fan-in to weight by. Damped is the default; raw is for auditing
    /// and benchmarking against the resolver measurement.
    #[arg(long, value_enum, default_value_t = Basis::Damped)]
    basis: Basis,
    /// Percentile at or above which a change is surfaced.
    #[arg(long, default_value_t = beholder::risk::DEFAULT_THRESHOLD)]
    threshold: f64,
    /// Emit the report structure instead of a rendering.
    #[arg(long)]
    json: bool,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Basis {
    Raw,
    Damped,
}

impl From<Basis> for beholder::risk::FanInBasis {
    fn from(basis: Basis) -> Self {
        match basis {
            Basis::Raw => Self::Raw,
            Basis::Damped => Self::Damped,
        }
    }
}

#[derive(Args)]
struct StoreArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// How many index commits to list.
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(Args)]
struct AuditArgs {
    /// Revision to walk back from.
    #[arg(default_value = "HEAD")]
    revision: String,
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// How many commits back to walk.
    #[arg(long, default_value_t = 30)]
    depth: usize,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Index(args) => index(args),
        Command::Delta(args) => run_delta(args),
        Command::Report(args) => report(args),
        Command::Store(args) => store(args),
        Command::AuditIdentity(args) => audit(args),
    }
}

fn open(root: &Path) -> Result<(git2::Repository, Config)> {
    let repo = git2::Repository::discover(root)
        .with_context(|| format!("{} is not inside a git repository", root.display()))?;
    let workdir = repo
        .workdir()
        .context("beholder needs a repository with a working tree")?
        .to_owned();
    let config = Config::load(&workdir)?;
    Ok((repo, config))
}

fn index(args: IndexArgs) -> Result<()> {
    let (repo, config) = open(&args.root)?;
    let workdir = repo.workdir().expect("checked in open").to_owned();
    let files = beholder::walk::walk_worktree(&workdir, &config)?;
    let fingerprint = Fingerprint::new(&config);

    let (analysis, stats) = if args.use_store {
        let store = Store::open(&repo);
        let cache = store.phase1_cache(&fingerprint)?;
        analysis::run_with_cache(&files, &config, &cache)
    } else {
        analysis::run_with_cache(&files, &config, &analysis::NoCache)
    };

    let out_dir = args.out_dir.unwrap_or_else(|| workdir.clone());
    std::fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;

    write_jsonl(
        &out_dir.join("symbols.jsonl"),
        &jsonl::symbol_records(&analysis),
    )?;
    write_jsonl(
        &out_dir.join("files.jsonl"),
        &jsonl::file_records(&analysis),
    )?;
    write_jsonl(
        &out_dir.join("edges.jsonl"),
        &jsonl::edge_records(&analysis),
    )?;

    if args.store {
        let head = repo.head()?.peel_to_commit()?.id();
        refuse_to_store_a_dirty_tree(&repo, head, &analysis, &config)?;
        let index_commit = Store::open(&repo).write(head, &analysis)?;
        println!("stored {index_commit} on {DEFAULT_REF}");
    }

    let graph = &analysis.phase2.graph;
    println!(
        "{} files, {} symbols, {} edges ({} reused, {} computed)",
        analysis.files.len(),
        analysis.symbols().count(),
        graph.edges.len(),
        stats.reused,
        stats.computed
    );
    println!(
        "resolved {} of {} occurrences",
        graph.resolution.resolved, graph.resolution.occurrences
    );
    for (language, accuracy) in &graph.accuracy {
        match (accuracy.precision_range, accuracy.recall_range) {
            (Some(precision), Some(recall)) => println!(
                "{language}: {resolver} resolver, precision {:.2}-{:.2}, recall {:.2}-{:.2} across {} repositories",
                precision.0,
                precision.1,
                recall.0,
                recall.1,
                accuracy.measurements.len(),
                resolver = accuracy.resolver,
            ),
            _ => println!(
                "{language}: {} resolver, accuracy not measured",
                accuracy.resolver
            ),
        }
    }

    Ok(())
}

/// Refuse to file a working-tree analysis under a commit it does not describe.
///
/// `index` analyzes what is on disk. Storing that against HEAD is only honest
/// when the two agree, and a stored result that is wrong is worse than no stored
/// result: every later run and every fresh clone would trust it.
fn refuse_to_store_a_dirty_tree(
    repo: &git2::Repository,
    head: git2::Oid,
    analysis: &beholder::Analysis,
    config: &Config,
) -> Result<()> {
    let manifest = beholder::walk::revision_manifest(repo, &head.to_string(), config)?;

    if let Err(divergence) = beholder::store::validate_payload(analysis, &manifest) {
        anyhow::bail!(
            "refusing to store an analysis of the working tree against {head}: {divergence}.\n\
             Commit the changes first, or drop --store to write JSONL only."
        );
    }

    Ok(())
}

fn write_jsonl<T: serde::Serialize>(path: &Path, records: &[T]) -> Result<()> {
    let file =
        std::fs::File::create(path).with_context(|| format!("creating {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);
    jsonl::write(&mut writer, records)?;
    writer.flush()?;
    Ok(())
}

fn run_delta(args: DeltaArgs) -> Result<()> {
    let (repo, config) = open(&args.root)?;
    let store = args.use_store.then(|| Store::open(&repo));

    let delta =
        delta::compare_revisions(&repo, &args.before, &args.after, &config, store.as_ref())?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for change in &delta.changes {
        serde_json::to_writer(&mut out, change)?;
        out.write_all(b"\n")?;
    }

    Ok(())
}

fn report(args: ReportArgs) -> Result<()> {
    let (repo, config) = open(&args.root)?;

    let before = delta::analyze_revision(&repo, &args.before, &config, None)?;
    let after = delta::analyze_revision(&repo, &args.after, &config, None)?;
    let changed = delta::compare(&before, &after);

    let report = beholder::risk::rank(
        &args.before,
        &args.after,
        &changed,
        &before,
        &after,
        args.basis.into(),
        args.threshold,
    );

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", beholder::risk::render(&report));
    }

    Ok(())
}

fn store(args: StoreArgs) -> Result<()> {
    let (repo, _) = open(&args.root)?;
    let store = Store::open(&repo);

    println!("ref:     {}", store.refname());
    println!("refspec: {REFSPEC}");

    match store.tip()? {
        None => println!("tip:     (no index yet)"),
        Some(tip) => println!("tip:     {tip}"),
    }

    for entry in store.history(args.limit)? {
        println!(
            "{} <- source {} (beholder {})",
            entry.id, entry.source_commit, entry.meta.fingerprint.tool_version
        );
    }

    Ok(())
}

fn audit(args: AuditArgs) -> Result<()> {
    let (repo, config) = open(&args.root)?;

    let mut walk = repo.revwalk()?;
    walk.push(repo.revparse_single(&args.revision)?.peel_to_commit()?.id())?;
    walk.set_sorting(git2::Sort::TOPOLOGICAL)?;
    // First parent only, so each adjacent pair really is one revision and the
    // one that followed it.
    walk.simplify_first_parent()?;

    let mut revisions: Vec<String> = walk
        .take(args.depth)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(|id| id.to_string())
        .collect();
    revisions.reverse();

    let audit = delta::audit_identity(&repo, &revisions, &config)?;

    println!("comparisons: {}", audit.comparisons);
    println!("symbols:     {}", audit.symbols_seen);
    println!("changes:     {}", audit.changes);
    println!(
        "  added {} removed {} modified {} moved {} renamed {}",
        audit.added, audit.removed, audit.modified, audit.moved, audit.renamed
    );
    println!("phantoms:    {}", audit.phantoms.len());

    for phantom in &audit.phantoms {
        let short = |r: &str| r.chars().take(8).collect::<String>();
        println!(
            "  {} -> {}: {:?} {}",
            short(&phantom.before_revision),
            short(&phantom.after_revision),
            phantom.change.change,
            phantom.change.paths().join(", ")
        );
    }

    if !audit.passed() {
        anyhow::bail!("{} phantom changes found", audit.phantoms.len());
    }

    Ok(())
}
