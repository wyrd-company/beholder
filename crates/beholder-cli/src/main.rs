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
    /// Render a report that has already been produced.
    ///
    /// A review surface wants the same report as SARIF and as a comment. The
    /// analysis is the expensive part, so it runs once, and every rendering
    /// comes from the report it produced.
    Render(RenderArgs),
    /// Report on a local gitpr review snapshot.
    Gitpr(GitprArgs),
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
    /// How to render the report.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    /// Read and append phase results in the beholder ref.
    #[arg(long)]
    use_store: bool,
    /// What to render when nothing crossed the threshold.
    #[arg(long, value_enum, default_value_t = WhenQuiet::Omit)]
    when_quiet: WhenQuiet,
}

#[derive(Args)]
struct RenderArgs {
    /// A report written by `beholder report --format json`. Reads stdin when
    /// this is `-` or absent.
    #[arg(long)]
    from: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long, value_enum, default_value_t = WhenQuiet::Omit)]
    when_quiet: WhenQuiet,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Format {
    /// For a person to read.
    Text,
    /// The surface-agnostic report structure.
    Json,
    /// SARIF 2.1.0, for inline annotations on a review surface.
    Sarif,
    /// A sticky pull request comment.
    Markdown,
}

/// What a surface should be handed when a report has nothing to say.
#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum WhenQuiet {
    /// Render nothing. Empty output is how a surface learns to stay silent.
    Omit,
    /// Render a body saying so, for a comment that already exists and would
    /// otherwise keep making a claim about code that has since changed.
    Note,
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
struct GitprArgs {
    /// Snapshot id. Defaults to the only open snapshot.
    pr: Option<String>,
    #[arg(long, default_value = ".")]
    root: PathBuf,
    #[arg(long, value_enum, default_value_t = Basis::Damped)]
    basis: Basis,
    #[arg(long, default_value_t = beholder::risk::DEFAULT_THRESHOLD)]
    threshold: f64,
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
    #[arg(long, value_enum, default_value_t = WhenQuiet::Omit)]
    when_quiet: WhenQuiet,
}

#[derive(Args)]
struct StoreArgs {
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// How many index commits to list.
    #[arg(long, default_value_t = 20)]
    limit: usize,
    /// Fetch the ref from this remote before reporting on it.
    ///
    /// A missing ref on the remote is not an error: a repository nobody has
    /// indexed yet is the ordinary cold start.
    #[arg(long, value_name = "REMOTE")]
    fetch: Option<String>,
    /// Publish the ref to this remote, never overwriting another writer.
    #[arg(long, value_name = "REMOTE")]
    push: Option<String>,
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
        Command::Render(args) => render(args),
        Command::Gitpr(args) => gitpr(args),
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
    let store = args.use_store.then(|| Store::open(&repo));

    let compared = delta::compare_revisions_detailed(
        &repo,
        &args.before,
        &args.after,
        &config,
        store.as_ref(),
    )?;

    if args.use_store {
        // On stdout the report is the output, so this goes to stderr — where a
        // CI log still shows whether fetching the index saved the run any work.
        eprintln!("{}: {}", short(&args.before), compared.origins.0);
        eprintln!("{}: {}", short(&args.after), compared.origins.1);
    }

    let report = beholder::risk::rank(
        &args.before,
        &args.after,
        &compared.delta,
        &compared.before,
        &compared.after,
        args.basis.into(),
        args.threshold,
    );

    emit(&report, args.format, args.when_quiet)
}

fn render(args: RenderArgs) -> Result<()> {
    let from = args.from.as_deref().filter(|path| *path != Path::new("-"));
    let text = match from {
        None => std::io::read_to_string(std::io::stdin().lock())
            .context("reading a report from stdin")?,
        Some(path) => {
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?
        }
    };

    let report: beholder::risk::Report =
        serde_json::from_str(&text).context("parsing a beholder report")?;

    emit(&report, args.format, args.when_quiet)
}

/// Render one report onto one surface.
///
/// Every surface consumes the same structure; the only thing a format decides
/// is how it reads. Silence is a rendering too: an empty markdown body is what
/// tells a review surface to post nothing.
fn emit(report: &beholder::risk::Report, format: Format, when_quiet: WhenQuiet) -> Result<()> {
    match format {
        Format::Text => print!("{}", beholder::risk::render(report)),
        Format::Json => println!("{}", serde_json::to_string_pretty(report)?),
        Format::Sarif => println!(
            "{}",
            serde_json::to_string_pretty(&beholder::sarif::render(report))?
        ),
        Format::Markdown => match beholder::markdown::render(report) {
            Some(body) => print!("{body}"),
            None if when_quiet == WhenQuiet::Note => {
                print!("{}", beholder::markdown::render_quiet(report))
            }
            None => {}
        },
    }

    Ok(())
}

/// Report on a gitpr snapshot.
///
/// The adapter's whole job is turning a snapshot into two revisions. Everything
/// after that is the same code path a GitHub pull request takes, rendering the
/// same report structure — which is the point of the structure being
/// surface-agnostic.
fn gitpr(args: GitprArgs) -> Result<()> {
    let snapshot = read_gitpr_snapshot(&args.root, args.pr.as_deref())?;

    let (repo, config) = open(&args.root)?;
    let before = delta::analyze_revision(&repo, &snapshot.merge_base, &config, None)?;
    let after = delta::analyze_revision(&repo, &snapshot.head, &config, None)?;
    let changed = delta::compare(&before, &after);

    let report = beholder::risk::rank(
        &snapshot.merge_base,
        &snapshot.head,
        &changed,
        &before,
        &after,
        args.basis.into(),
        args.threshold,
    );

    if let Format::Text = args.format {
        println!("gitpr {} — {}", snapshot.id, snapshot.title);
    }

    emit(&report, args.format, args.when_quiet)
}

struct Snapshot {
    id: String,
    title: String,
    merge_base: String,
    head: String,
}

/// Read the revisions a gitpr snapshot describes.
///
/// `gitpr show` emits YAML whose leading block is flat scalars. Only four of
/// them are needed, so they are read directly rather than taking on a YAML
/// parser for the privilege.
fn read_gitpr_snapshot(root: &Path, pr: Option<&str>) -> Result<Snapshot> {
    let id = match pr {
        Some(id) => id.to_owned(),
        None => only_open_snapshot(root)?,
    };

    let output = std::process::Command::new("gitpr")
        .arg("show")
        .arg(&id)
        .current_dir(root)
        .output()
        .context("running `gitpr show`; is gitpr installed?")?;

    anyhow::ensure!(
        output.status.success(),
        "gitpr show {id} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let text = String::from_utf8(output.stdout).context("gitpr show emitted invalid UTF-8")?;
    let field = |key: &str| {
        text.lines()
            .take_while(|line| !line.starts_with("file_diffs:"))
            .find_map(|line| line.strip_prefix(&format!("{key}: ")))
            .map(str::to_owned)
    };

    Ok(Snapshot {
        title: field("title").unwrap_or_else(|| "(untitled)".to_owned()),
        merge_base: field("merge_base_sha")
            .or_else(|| field("base_head_sha"))
            .with_context(|| format!("snapshot {id} names no base revision"))?,
        head: field("source_head_sha")
            .with_context(|| format!("snapshot {id} names no head revision"))?,
        id,
    })
}

fn only_open_snapshot(root: &Path) -> Result<String> {
    let output = std::process::Command::new("gitpr")
        .arg("list")
        .arg("--status")
        .arg("open")
        .current_dir(root)
        .output()
        .context("running `gitpr list`")?;

    // `gitpr list` prints a header row and then one snapshot per line, whose
    // first column is the id — abbreviated, which `gitpr show` accepts.
    let text = String::from_utf8_lossy(&output.stdout);
    let ids: Vec<&str> = text
        .lines()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    match ids.as_slice() {
        [only] => Ok((*only).to_owned()),
        [] => anyhow::bail!("no open gitpr snapshot; name one explicitly"),
        many => anyhow::bail!("{} open gitpr snapshots; name one explicitly", many.len()),
    }
}

fn store(args: StoreArgs) -> Result<()> {
    let (repo, _) = open(&args.root)?;
    let mut store = Store::open(&repo);

    if let Some(remote) = &args.fetch {
        // A remote nobody has indexed yet matches the wildcard refspec with
        // nothing and succeeds, which is the cold start. Anything that does
        // fail here — an unreachable remote, refused credentials — is a fault,
        // and reporting it as an empty index would hide it forever behind work
        // that merely looks slow.
        store
            .fetch(remote)
            .with_context(|| format!("fetching the stored index from {remote}"))?;

        match store.tip()? {
            Some(_) => println!("fetched {REFSPEC} from {remote}"),
            None => println!("cold start: {remote} has no stored index yet"),
        }
    }

    if let Some(remote) = &args.push {
        store = store.with_remote(remote.clone());
        match store.tip()? {
            None => println!("nothing to push to {remote}: no index yet"),
            Some(_) => {
                store.push(remote)?;
                println!("pushed {} to {remote}", store.refname());
            }
        }
    }

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

fn short(revision: &str) -> String {
    if revision.len() > 8 && revision.chars().all(|c| c.is_ascii_hexdigit()) {
        revision.chars().take(8).collect()
    } else {
        revision.to_owned()
    }
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
