use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use mni::{Minifier, MinifyOptions, Target};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mni")]
#[command(about = "A world-class minifier for JavaScript, CSS, and JSON", long_about = None)]
#[command(version)]
struct Cli {
    /// Input file (use - for stdin)
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file (use - for stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

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
    let cli = Cli::parse();

    // Read input
    let (source, filename) = read_input(&cli.input)?;

    // Build options
    let mut options = match cli.preset {
        Some(Preset::Dev) => MinifyOptions::development(),
        Some(Preset::Prod) => MinifyOptions::production(),
        Some(Preset::Aggressive) => MinifyOptions::aggressive(),
        None => MinifyOptions::default(),
    };

    // Override with CLI options
    options.target = cli.target.into();
    options.mangle = cli.mangle;
    options.compress = cli.compress;
    options.source_map = cli.source_map;
    options.keep_fnames = cli.keep_fnames;
    options.keep_classnames = cli.keep_classnames;
    options.compress_options.drop_console = cli.drop_console;
    options.compress_options.drop_debugger = cli.drop_debugger;
    options.compress_options.passes = cli.passes;

    // Minify
    let minifier = Minifier::new(options);
    let result = minifier
        .minify_auto(&source, filename.as_deref())
        .context("Minification failed")?;

    // Write output
    write_output(&cli.output, &result.code)?;

    // Write source map if requested
    if cli.source_map
        && let Some(map) = &result.map
    {
        let map_path = match &cli.output {
            Some(path) if path.to_str() != Some("-") => {
                let mut map_path = path.clone();
                map_path.set_extension(format!(
                    "{}.map",
                    path.extension().and_then(|s| s.to_str()).unwrap_or("js")
                ));
                map_path
            }
            _ => PathBuf::from("output.map"),
        };
        fs::write(&map_path, map)
            .with_context(|| format!("Failed to write source map to {}", map_path.display()))?;
    }

    // Show statistics
    if cli.stats {
        eprintln!("\nMinification Statistics:");
        eprintln!("  Original:  {} bytes", result.stats.original_size);
        eprintln!("  Minified:  {} bytes", result.stats.minified_size);
        eprintln!(
            "  Reduction: {:.1}%",
            result.stats.compression_ratio * 100.0
        );
        eprintln!("  Time:      {} ms", result.stats.time_ms);
    }

    Ok(())
}

fn read_input(path: &Option<PathBuf>) -> Result<(String, Option<String>)> {
    match path {
        Some(path) if path.to_str() == Some("-") || path.to_str() == Some("") => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            Ok((buffer, None))
        }
        Some(path) => {
            // Read from file
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            Ok((content, Some(path.to_string_lossy().to_string())))
        }
        None => {
            // Default to stdin
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
        Some(path) if path.to_str() == Some("-") => {
            // Write to stdout
            io::stdout()
                .write_all(content.as_bytes())
                .context("Failed to write to stdout")
        }
        Some(path) => {
            // Write to file
            fs::write(path, content)
                .with_context(|| format!("Failed to write to file: {}", path.display()))
        }
        None => {
            // Default to stdout
            io::stdout()
                .write_all(content.as_bytes())
                .context("Failed to write to stdout")
        }
    }
}
