use anyhow::{Context, Result};
use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, ValueEnum};
use mni::{Minifier, MinifyOptions, MinifyStats, Target};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Parser)]
#[command(name = "mni")]
#[command(about = "A world-class minifier for JavaScript, CSS, and JSON", long_about = None)]
#[command(version)]
struct Cli {
    /// Input file(s). Use `-` or omit for stdin. Multiple files require --outdir.
    #[arg(value_name = "FILE")]
    inputs: Vec<PathBuf>,

    /// Output file for single-file mode (use - for stdout)
    #[arg(short, long, value_name = "FILE", conflicts_with = "outdir")]
    output: Option<PathBuf>,

    /// Output directory for batch mode. Required when multiple inputs are given.
    #[arg(long, value_name = "DIR")]
    outdir: Option<PathBuf>,

    /// Target ECMAScript version
    #[arg(short, long, value_enum, default_value = "es2020")]
    target: TargetArg,

    /// Enable identifier mangling
    #[arg(short, long, default_value = "true")]
    mangle: bool,

    /// Enable compression optimizations
    #[arg(short, long, default_value = "true")]
    compress: bool,

    /// Generate source map
    #[arg(long)]
    source_map: bool,

    /// Keep function names
    #[arg(long)]
    keep_fnames: bool,

    /// Keep class names
    #[arg(long)]
    keep_classnames: bool,

    /// Drop console statements
    #[arg(long)]
    drop_console: bool,

    /// Drop debugger statements
    #[arg(long, default_value = "true")]
    drop_debugger: bool,

    /// Number of compression passes
    #[arg(long, default_value = "1")]
    passes: usize,

    /// Use preset configuration
    #[arg(short, long, value_enum)]
    preset: Option<Preset>,

    /// Show statistics
    #[arg(long)]
    stats: bool,

    /// Disable parallel processing in batch mode (sequential fallback)
    #[arg(long)]
    no_parallel: bool,

    /// Load options from a JSON config file. If omitted, looks for
    /// `.minirc.json` or `mni.config.json` in the current directory.
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Skip config file auto-discovery (overrides default .minirc.json lookup)
    #[arg(long)]
    no_config: bool,

    /// Watch input files and re-run minification on change
    #[arg(long)]
    watch: bool,
}

#[derive(Clone, ValueEnum)]
enum TargetArg {
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    ES2024,
    ESNext,
}

impl From<TargetArg> for Target {
    fn from(arg: TargetArg) -> Self {
        match arg {
            TargetArg::ES5 => Target::ES5,
            TargetArg::ES2015 => Target::ES2015,
            TargetArg::ES2016 => Target::ES2016,
            TargetArg::ES2017 => Target::ES2017,
            TargetArg::ES2018 => Target::ES2018,
            TargetArg::ES2019 => Target::ES2019,
            TargetArg::ES2020 => Target::ES2020,
            TargetArg::ES2021 => Target::ES2021,
            TargetArg::ES2022 => Target::ES2022,
            TargetArg::ES2023 => Target::ES2023,
            TargetArg::ES2024 => Target::ES2024,
            TargetArg::ESNext => Target::ESNext,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum Preset {
    /// Development preset (fast, readable)
    Dev,
    /// Production preset (balanced)
    Prod,
    /// Aggressive preset (maximum compression)
    Aggressive,
}

fn main() -> Result<()> {
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches)
        .map_err(|e| anyhow::anyhow!("failed to parse arguments: {e}"))?;
    let options = build_options(&cli, &matches)?;
    let minifier = Minifier::new(options);

    if cli.watch {
        return run_watch(&cli, &minifier);
    }

    rebuild_once(&cli, &minifier)
}

fn rebuild_once(cli: &Cli, minifier: &Minifier) -> Result<()> {
    match cli.inputs.len() {
        0 | 1 if cli.outdir.is_none() => run_single(cli, minifier),
        _ => run_batch(cli, minifier),
    }
}

fn run_watch(cli: &Cli, minifier: &Minifier) -> Result<()> {
    if cli.inputs.is_empty() {
        anyhow::bail!("--watch requires at least one input file");
    }

    // Initial build before wiring up the watcher so the user sees output fast.
    if let Err(err) = rebuild_once(cli, minifier) {
        eprintln!("initial build failed: {err:#}");
    }

    let (tx, rx) = std::sync::mpsc::channel::<WatchSignal>();
    let notify_tx = tx.clone();

    // Spawn a debouncing bridge: forward notify events into WatchSignal::Rebuild,
    // coalescing bursts within DEBOUNCE_MS.
    let (event_tx, event_rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = event_tx.send(res);
    })
    .context("failed to create file watcher")?;

    use notify::Watcher;
    for input in &cli.inputs {
        watcher
            .watch(input, notify::RecursiveMode::NonRecursive)
            .with_context(|| format!("failed to watch {}", input.display()))?;
    }

    std::thread::spawn(move || {
        const DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(150);
        let mut pending = false;
        loop {
            let next = if pending {
                event_rx.recv_timeout(DEBOUNCE)
            } else {
                event_rx
                    .recv()
                    .map_err(|_| std::sync::mpsc::RecvTimeoutError::Disconnected)
            };
            match next {
                Ok(Ok(event)) if is_content_event(&event) => {
                    pending = true;
                }
                Ok(Ok(_)) => {}
                Ok(Err(err)) => {
                    eprintln!("watcher error: {err}");
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) if pending => {
                    pending = false;
                    if notify_tx.send(WatchSignal::Rebuild).is_err() {
                        return;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
            }
        }
    });

    eprintln!("watching {} input(s); Ctrl-C to stop", cli.inputs.len());

    run_watch_loop(&rx, || rebuild_once(cli, minifier))
}

fn is_content_event(event: &notify::Event) -> bool {
    use notify::EventKind;
    matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
    )
}

fn build_options(cli: &Cli, matches: &ArgMatches) -> Result<MinifyOptions> {
    // 1. Start from preset (or built-in defaults).
    let mut options = match cli.preset {
        Some(Preset::Dev) => MinifyOptions::development(),
        Some(Preset::Prod) => MinifyOptions::production(),
        Some(Preset::Aggressive) => MinifyOptions::aggressive(),
        None => MinifyOptions::default(),
    };

    // 2. Overlay a config file, if one is available (explicit path wins over
    //    auto-discovery).
    if let Some(path) = resolve_config_path(cli) {
        options = merge_config_file(options, &path)
            .with_context(|| format!("Failed to load config file {}", path.display()))?;
    }

    // 3. CLI flags override — but only when the user actually typed them. This
    //    lets config files set `mangle: false` without clap's `default_value`
    //    silently clobbering it back to true.
    if is_explicit(matches, "target") {
        options.target = cli.target.clone().into();
    }
    if is_explicit(matches, "mangle") {
        options.mangle = cli.mangle;
    }
    if is_explicit(matches, "compress") {
        options.compress = cli.compress;
    }
    if is_explicit(matches, "source_map") {
        options.source_map = cli.source_map;
    }
    if is_explicit(matches, "keep_fnames") {
        options.keep_fnames = cli.keep_fnames;
    }
    if is_explicit(matches, "keep_classnames") {
        options.keep_classnames = cli.keep_classnames;
    }
    if is_explicit(matches, "drop_console") {
        options.compress_options.drop_console = cli.drop_console;
    }
    if is_explicit(matches, "drop_debugger") {
        options.compress_options.drop_debugger = cli.drop_debugger;
    }
    if is_explicit(matches, "passes") {
        options.compress_options.passes = cli.passes;
    }
    if is_explicit(matches, "no_parallel") {
        options.parallel = !cli.no_parallel;
    }

    Ok(options)
}

fn is_explicit(matches: &ArgMatches, name: &str) -> bool {
    matches.value_source(name) == Some(ValueSource::CommandLine)
}

fn resolve_config_path(cli: &Cli) -> Option<PathBuf> {
    if cli.no_config {
        return None;
    }
    if let Some(path) = cli.config.as_ref() {
        return Some(path.clone());
    }
    for candidate in [".minirc.json", "mni.config.json"] {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn merge_config_file(base: MinifyOptions, path: &Path) -> Result<MinifyOptions> {
    let text =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let overlay: serde_json::Value =
        serde_json::from_str(&text).context("Config file is not valid JSON")?;
    let mut merged = serde_json::to_value(&base).context("Failed to serialize base options")?;
    merge_json(&mut merged, &overlay);
    serde_json::from_value(merged).context("Config file produced an invalid MinifyOptions")
}

fn merge_json(base: &mut serde_json::Value, overlay: &serde_json::Value) {
    match (base.as_object_mut(), overlay.as_object()) {
        (Some(b), Some(o)) => {
            for (k, v) in o {
                if let Some(existing) = b.get_mut(k) {
                    merge_json(existing, v);
                } else {
                    b.insert(k.clone(), v.clone());
                }
            }
        }
        _ => {
            *base = overlay.clone();
        }
    }
}

// ---------- Single-file mode ----------

fn run_single(cli: &Cli, minifier: &Minifier) -> Result<()> {
    let input_path = cli.inputs.first();
    let (source, filename) = read_source(input_path)?;

    let result = minifier
        .minify_auto(&source, filename.as_deref())
        .context("Minification failed")?;

    let (final_code, map_path_to_write) = prepare_source_map_output(
        &cli.output,
        cli.source_map,
        &result.code,
        result.map.as_ref(),
    );

    write_output(&cli.output, &final_code)?;

    if let (Some(map_path), Some(map)) = (map_path_to_write, result.map.as_ref()) {
        fs::write(&map_path, map)
            .with_context(|| format!("Failed to write source map to {}", map_path.display()))?;
    }

    if cli.stats {
        print_stats_single(&result.stats);
    }

    Ok(())
}

fn read_source(input: Option<&PathBuf>) -> Result<(String, Option<String>)> {
    match input {
        Some(path) if path.to_str() == Some("-") => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            Ok((buffer, None))
        }
        Some(path) => {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            Ok((content, Some(path.to_string_lossy().into_owned())))
        }
        None => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            Ok((buffer, None))
        }
    }
}

fn write_output(path: &Option<PathBuf>, content: &str) -> Result<()> {
    match path {
        Some(path) if path.to_str() == Some("-") => io::stdout()
            .write_all(content.as_bytes())
            .context("Failed to write to stdout"),
        Some(path) => fs::write(path, content)
            .with_context(|| format!("Failed to write to file: {}", path.display())),
        None => io::stdout()
            .write_all(content.as_bytes())
            .context("Failed to write to stdout"),
    }
}

// ---------- Batch mode ----------

fn run_batch(cli: &Cli, minifier: &Minifier) -> Result<()> {
    let outdir = cli
        .outdir
        .as_ref()
        .context("multiple inputs require --outdir <DIR>")?;

    fs::create_dir_all(outdir)
        .with_context(|| format!("Failed to create output directory {}", outdir.display()))?;

    let errors: Mutex<Vec<(PathBuf, anyhow::Error)>> = Mutex::new(vec![]);
    let rows: Mutex<Vec<BatchStatsRow>> = Mutex::new(vec![]);

    let work = |input: &PathBuf| match process_one(input, outdir, cli.source_map, minifier) {
        Ok(stats) => {
            rows.lock().unwrap().push(BatchStatsRow {
                path: input.clone(),
                stats,
            });
        }
        Err(err) => {
            errors.lock().unwrap().push((input.clone(), err));
        }
    };

    if cli.no_parallel {
        cli.inputs.iter().for_each(work);
    } else {
        cli.inputs.par_iter().for_each(work);
    }

    let errors = errors.into_inner().unwrap();
    let mut rows = rows.into_inner().unwrap();
    rows.sort_by(|a, b| a.path.cmp(&b.path));
    let processed = rows.len();

    let mut totals = MinifyStats::default();
    for row in &rows {
        totals.original_size += row.stats.original_size;
        totals.minified_size += row.stats.minified_size;
        totals.time_ms += row.stats.time_ms;
    }
    if processed > 0 && totals.original_size > 0 {
        #[allow(clippy::cast_precision_loss)]
        {
            totals.compression_ratio =
                1.0 - (totals.minified_size as f64 / totals.original_size as f64);
        }
    }

    for (path, err) in &errors {
        eprintln!("error: {}: {err:#}", path.display());
    }

    if cli.stats {
        print_stats_batch(&rows, processed, errors.len(), &totals);
    }

    if !errors.is_empty() {
        anyhow::bail!("{} file(s) failed to minify", errors.len());
    }

    Ok(())
}

fn process_one(
    input: &Path,
    outdir: &Path,
    source_map_enabled: bool,
    minifier: &Minifier,
) -> Result<MinifyStats> {
    let source =
        fs::read_to_string(input).with_context(|| format!("Failed to read {}", input.display()))?;
    let filename_str = input.to_string_lossy().into_owned();

    let result = minifier
        .minify_auto(&source, Some(&filename_str))
        .context("Minification failed")?;

    let out_path = outdir.join(input.file_name().context("input has no file name")?);
    let out_path_wrapper = Some(out_path.clone());
    let (final_code, map_path_to_write) = prepare_source_map_output(
        &out_path_wrapper,
        source_map_enabled,
        &result.code,
        result.map.as_ref(),
    );

    fs::write(&out_path, &final_code)
        .with_context(|| format!("Failed to write {}", out_path.display()))?;

    if let (Some(map_path), Some(map)) = (map_path_to_write, result.map.as_ref()) {
        fs::write(&map_path, map)
            .with_context(|| format!("Failed to write source map to {}", map_path.display()))?;
    }

    Ok(result.stats)
}

// ---------- Source map wiring ----------

/// Decide what to write for the minified code and (optionally) where the `.map`
/// file should live. Returns the code — possibly with a `sourceMappingURL`
/// footer appended — and the map path to write (if any).
fn prepare_source_map_output(
    output: &Option<PathBuf>,
    source_map_enabled: bool,
    code: &str,
    map: Option<&String>,
) -> (String, Option<PathBuf>) {
    if !source_map_enabled || map.is_none() {
        return (code.to_string(), None);
    }

    let Some(path) = output else {
        eprintln!(
            "warning: --source-map requires a real output file; source map not written to disk"
        );
        return (code.to_string(), None);
    };
    if path.to_str() == Some("-") {
        eprintln!(
            "warning: --source-map requires a real output file; source map not written to disk"
        );
        return (code.to_string(), None);
    }

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let map_file_name = format!("{file_name}.map");
    let mut map_path = path.clone();
    map_path.set_file_name(&map_file_name);

    let is_css = path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("css"));

    let footer = if is_css {
        format!("\n/*# sourceMappingURL={map_file_name} */\n")
    } else {
        format!("\n//# sourceMappingURL={map_file_name}\n")
    };

    let mut final_code = String::with_capacity(code.len() + footer.len());
    final_code.push_str(code);
    final_code.push_str(&footer);

    (final_code, Some(map_path))
}

// ---------- Watch mode ----------

/// Signal delivered to the watch loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatchSignal {
    Rebuild,
    /// Only used by unit tests today; Ctrl-C kills the real process before
    /// the loop observes a shutdown signal.
    #[allow(dead_code)]
    Shutdown,
}

/// Pure rebuild-on-signal loop. Factored out so it's unit-testable without
/// needing a real filesystem watcher.
fn run_watch_loop<F>(rx: &std::sync::mpsc::Receiver<WatchSignal>, mut rebuild: F) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    while let Ok(signal) = rx.recv() {
        match signal {
            WatchSignal::Rebuild => {
                if let Err(err) = rebuild() {
                    eprintln!("rebuild failed: {err:#}");
                }
            }
            WatchSignal::Shutdown => return Ok(()),
        }
    }
    Ok(())
}

// ---------- Stats ----------

fn print_stats_single(stats: &MinifyStats) {
    eprintln!("\nMinification Statistics:");
    eprintln!("  Original:  {} bytes", stats.original_size);
    eprintln!("  Minified:  {} bytes", stats.minified_size);
    eprintln!("  Reduction: {:.1}%", stats.compression_ratio * 100.0);
    eprintln!("  Time:      {} ms", stats.time_ms);
}

fn print_stats_batch(
    rows: &[BatchStatsRow],
    processed: usize,
    failed: usize,
    totals: &MinifyStats,
) {
    eprintln!("{}", format_batch_stats_table(rows));
    eprintln!("Batch Minification Statistics:");
    eprintln!("  Files processed: {processed}");
    if failed > 0 {
        eprintln!("  Files failed:    {failed}");
    }
    eprintln!("  Original total:  {} bytes", totals.original_size);
    eprintln!("  Minified total:  {} bytes", totals.minified_size);
    eprintln!(
        "  Reduction:       {:.1}%",
        totals.compression_ratio * 100.0
    );
    eprintln!("  Wall time (sum): {} ms", totals.time_ms);
}

#[derive(Debug, Clone)]
struct BatchStatsRow {
    path: PathBuf,
    stats: MinifyStats,
}

/// Render the per-file stats table. Pure function — unit tested below.
fn format_batch_stats_table(rows: &[BatchStatsRow]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    const NAME_H: &str = "File";
    const ORIG_H: &str = "Original";
    const MIN_H: &str = "Minified";
    const RED_H: &str = "Reduction";
    const TIME_H: &str = "Time";

    let name_w = rows
        .iter()
        .map(|r| r.path.display().to_string().len())
        .max()
        .unwrap_or(0)
        .max(NAME_H.len());
    let orig_w = rows
        .iter()
        .map(|r| format!("{} B", r.stats.original_size).len())
        .max()
        .unwrap_or(0)
        .max(ORIG_H.len());
    let min_w = rows
        .iter()
        .map(|r| format!("{} B", r.stats.minified_size).len())
        .max()
        .unwrap_or(0)
        .max(MIN_H.len());
    let red_w = rows
        .iter()
        .map(|r| format!("{:.1}%", r.stats.compression_ratio * 100.0).len())
        .max()
        .unwrap_or(0)
        .max(RED_H.len());
    let time_w = rows
        .iter()
        .map(|r| format!("{} ms", r.stats.time_ms).len())
        .max()
        .unwrap_or(0)
        .max(TIME_H.len());

    let mut out = String::new();
    out.push_str(&format!(
        "  {:<name_w$}  {:>orig_w$}  {:>min_w$}  {:>red_w$}  {:>time_w$}\n",
        NAME_H,
        ORIG_H,
        MIN_H,
        RED_H,
        TIME_H,
        name_w = name_w,
        orig_w = orig_w,
        min_w = min_w,
        red_w = red_w,
        time_w = time_w,
    ));
    out.push_str(&format!(
        "  {:-<name_w$}  {:-<orig_w$}  {:-<min_w$}  {:-<red_w$}  {:-<time_w$}\n",
        "",
        "",
        "",
        "",
        "",
        name_w = name_w,
        orig_w = orig_w,
        min_w = min_w,
        red_w = red_w,
        time_w = time_w,
    ));

    for row in rows {
        let name = row.path.display().to_string();
        let orig = format!("{} B", row.stats.original_size);
        let min = format!("{} B", row.stats.minified_size);
        let red = format!("{:.1}%", row.stats.compression_ratio * 100.0);
        let time = format!("{} ms", row.stats.time_ms);
        out.push_str(&format!(
            "  {name:<name_w$}  {orig:>orig_w$}  {min:>min_w$}  {red:>red_w$}  {time:>time_w$}\n",
            name_w = name_w,
            orig_w = orig_w,
            min_w = min_w,
            red_w = red_w,
            time_w = time_w,
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;

    fn row(path: &str, original: usize, minified: usize, time_ms: u128) -> BatchStatsRow {
        BatchStatsRow {
            path: PathBuf::from(path),
            stats: MinifyStats::with_sizes(original, minified).with_time(time_ms),
        }
    }

    #[test]
    fn run_watch_loop_invokes_rebuild_per_signal_then_exits_on_shutdown() {
        let (tx, rx) = mpsc::channel::<WatchSignal>();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let handle = std::thread::spawn(move || {
            run_watch_loop(&rx, move || {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });

        tx.send(WatchSignal::Rebuild).unwrap();
        tx.send(WatchSignal::Rebuild).unwrap();
        tx.send(WatchSignal::Rebuild).unwrap();
        tx.send(WatchSignal::Shutdown).unwrap();

        handle.join().unwrap().unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn run_watch_loop_continues_after_rebuild_error() {
        let (tx, rx) = mpsc::channel::<WatchSignal>();
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let handle = std::thread::spawn(move || {
            run_watch_loop(&rx, move || {
                let n = count_clone.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Err(anyhow::anyhow!("first rebuild fails"))
                } else {
                    Ok(())
                }
            })
        });

        tx.send(WatchSignal::Rebuild).unwrap();
        tx.send(WatchSignal::Rebuild).unwrap();
        tx.send(WatchSignal::Shutdown).unwrap();

        handle.join().unwrap().unwrap();
        assert_eq!(
            count.load(Ordering::SeqCst),
            2,
            "loop should survive a rebuild error"
        );
    }

    #[test]
    fn run_watch_loop_exits_when_channel_disconnects() {
        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&count);

        let handle = std::thread::spawn(move || {
            let (tx, rx) = mpsc::channel::<WatchSignal>();
            tx.send(WatchSignal::Rebuild).unwrap();
            drop(tx);
            run_watch_loop(&rx, move || {
                count_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });

        handle.join().unwrap().unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn format_batch_stats_table_empty() {
        assert_eq!(format_batch_stats_table(&[]), "");
    }

    #[test]
    fn format_batch_stats_table_renders_header_and_rows() {
        let rows = vec![
            row("a.js", 1000, 500, 12),
            row("sub/long-name.css", 2000, 1400, 5),
        ];
        let out = format_batch_stats_table(&rows);

        // Header row must be present.
        assert!(out.contains("File"));
        assert!(out.contains("Original"));
        assert!(out.contains("Minified"));
        assert!(out.contains("Reduction"));
        assert!(out.contains("Time"));

        // Both file names appear.
        assert!(out.contains("a.js"));
        assert!(out.contains("sub/long-name.css"));

        // Formatted values appear.
        assert!(out.contains("1000 B"));
        assert!(out.contains("500 B"));
        assert!(out.contains("50.0%"));
        assert!(out.contains("12 ms"));
        assert!(out.contains("2000 B"));
        assert!(out.contains("30.0%"));
    }

    #[test]
    fn format_batch_stats_table_columns_align() {
        let rows = vec![row("a.js", 10, 5, 1), row("longer.css", 100, 50, 2)];
        let out = format_batch_stats_table(&rows);
        let lines: Vec<&str> = out.lines().collect();
        // Header + separator + 2 data rows
        assert_eq!(lines.len(), 4);
        // All rows are the same width (fixed-width columns).
        let w = lines[0].len();
        for line in &lines[1..] {
            assert_eq!(line.len(), w, "row width mismatch:\n{out}");
        }
    }
}
