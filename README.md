# mni - A World-Class Minifier

[![GitHub](https://img.shields.io/badge/GitHub-epistates/mni-blue?logo=github)](https://github.com/epistates/mni)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/epistates/mni/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/mni.svg)](https://crates.io/crates/mni)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-Active-brightgreen)](https://github.com/epistates/mni)

A blazing-fast, production-ready minifier for JavaScript, CSS, JSON, HTML, and SVG written in Rust.

Built on industry-leading libraries:
- **JavaScript**: SWC (powers Next.js, Deno, Vercel)
- **CSS**: LightningCSS (100x faster than cssnano, powers Parcel)
- **HTML**: minify-html (also minifies inline CSS/JS)
- **SVG**: oxvg (Rust port of SVGO, correctness-first `safe` preset)
- **JSON**: serde_json (Rust standard)

## Features

- **Production-Ready**: Built on battle-tested libraries used in production by millions
- **Blazing Fast**: SWC + LightningCSS performance (7x faster than Terser)
- **High Compression**: 30-45% compression with correctness guarantees
- **Safe by Default**: Conservative optimizations that never produce invalid code
- **Multi-format**: JavaScript (ES5-ESNext), CSS, JSON, HTML, SVG
- **Smart Detection**: Auto-detects file format
- **Source Maps**: JS (via SWC) and CSS (via LightningCSS)
- **Batch Mode**: Parallel multi-file minification with `--outdir`
- **Watch Mode**: `--watch` re-runs on filesystem changes
- **Config Files**: `.minirc.json` partial overlay with CLI override precedence
- **Rich CLI**: Comprehensive command-line interface
- **Configurable**: Presets and fine-grained control

## Installation

```bash
cargo install mni
```

Or build from source:

```bash
git clone https://github.com/epistates/mni
cd mni
cargo build --release
```

## Usage

### Basic Usage

```bash
# Minify JavaScript
mni input.js -o output.min.js

# Minify CSS
mni styles.css -o styles.min.css

# Minify JSON
mni data.json -o data.min.json

# Auto-detect format and show stats
mni input.js --stats
```

### From stdin/stdout

```bash
# Read from stdin, write to stdout
echo "const x = 1 + 2;" | mni

# Pipe through mni
cat input.js | mni > output.min.js
```

### Presets

```bash
# Development preset (fast, readable, source maps)
mni input.js --preset dev -o output.js

# Production preset (balanced compression and speed)
mni input.js --preset prod -o output.js

# Aggressive preset (maximum compression, slower)
mni input.js --preset aggressive -o output.js
```

### Advanced Options

```bash
# Customize minification
mni input.js \
  --target es2020 \
  --mangle \
  --compress \
  --drop-console \
  --keep-fnames \
  --passes 2 \
  --stats
```

### Batch Mode

```bash
# Minify many files in parallel into an output directory
mni src/a.js src/b.js src/c.css --outdir dist --source-map --stats

# Disable parallelism (sequential fallback)
mni src/*.js --outdir dist --no-parallel
```

Batch mode requires `--outdir`. Each input file is written to `<outdir>/<basename>`
along with a sibling `.map` file when `--source-map` is enabled. Failed files are
reported individually and the process exits non-zero if any file errored. With
`--stats`, a per-file table of sizes/reductions/times is printed alongside totals.

### Watch Mode

```bash
# Re-run minification whenever an input changes
mni src/app.js --outdir dist --watch
mni src/*.{js,css,html} --outdir dist --watch --source-map
```

`--watch` performs an initial build, then monitors the input files for changes
using a platform-native filesystem watcher (via `notify`). Rebuild events are
debounced (150ms) to coalesce bursts from editors that save in multiple steps.
Errors during rebuilds are reported but do not stop the watcher — press Ctrl-C
to exit.

### Config File

`mni` will auto-discover `.minirc.json` (or `mni.config.json`) in the current
directory, or you can point at one explicitly with `--config <path>`. The file
is a partial JSON overlay of `MinifyOptions` — you only specify the fields you
want to change.

```json
{
  "keep_fnames": true,
  "compress_options": {
    "drop_console": true,
    "passes": 2
  }
}
```

Precedence (lowest to highest): built-in defaults → `--preset` → config file →
explicit CLI flags. Use `--no-config` to skip auto-discovery.

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--target` | ECMAScript version (es5, es2015, es2020, esnext) | es2020 |
| `--mangle` | Enable identifier mangling | true |
| `--compress` | Enable compression optimizations | true |
| `--source-map` | Generate source maps | false |
| `--keep-fnames` | Preserve function names | false |
| `--keep-classnames` | Preserve class names | false |
| `--drop-console` | Remove console.* statements | false |
| `--drop-debugger` | Remove debugger statements | true |
| `--passes` | Number of compression passes | 1 |
| `--stats` | Show minification statistics | false |

## Library Usage

```rust
use mni::{Minifier, MinifyOptions, Target};

fn main() -> anyhow::Result<()> {
    let code = r#"
        function hello(name) {
            console.log("Hello, " + name + "!");
        }
    "#;

    let options = MinifyOptions {
        target: Target::ES2020,
        mangle: true,
        compress: true,
        ..Default::default()
    };

    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code)?;

    println!("Original: {} bytes", result.stats.original_size);
    println!("Minified: {} bytes", result.stats.minified_size);
    println!("Reduction: {:.1}%", result.stats.compression_ratio * 100.0);
    println!("Time: {} ms", result.stats.time_ms);
    println!("\n{}", result.code);

    Ok(())
}
```

## Performance

Benchmarks on real-world files:

| Format | Original | Minified | Reduction | Time |
|--------|----------|----------|-----------|------|
| JavaScript (1.3KB) | 1,307 bytes | 757 bytes | 42.1% | 18ms |
| CSS (1.7KB) | 1,657 bytes | 1,218 bytes | 26.5% | 6ms |
| JSON (671B) | 671 bytes | 519 bytes | 22.7% | <1ms |

### Comparison with Other Tools

mni leverages SWC which is:
- **7x faster** than Terser
- **20x faster** than Babel on single thread
- **70x faster** than Babel on 4 cores

CSS minification via LightningCSS is:
- **100x faster** than cssnano
- **2.7+ million lines/sec** throughput

## Optimization Techniques

### JavaScript (via SWC)
- Identifier mangling with frequency analysis
- Dead code elimination
- Constant folding and propagation
- Boolean and comparison optimizations
- Unreachable code removal
- Unused variable elimination
- Scope hoisting
- Arrow function optimization
- Template literal optimization

### CSS (via LightningCSS)
- Whitespace removal
- Comment removal
- Color minification (#ffffff → #fff)
- Length optimization (0px → 0)
- Property merging
- Vendor prefix optimization
- Calc() optimization
- Custom property optimization

### HTML (via minify-html)
- Whitespace and comment removal
- Attribute minification
- Inline `<style>` minification (delegates to LightningCSS)
- Inline `<script>` minification (delegates to minify-js)
- DOCTYPE and optional tag omission

### SVG (via oxvg — Rust SVGO port)
Uses oxvg's correctness-first `safe` preset, which skips transformations that
can visually change the document. Typical optimizations applied:
- Whitespace, comment, metadata, and editor-namespace stripping
- Shape → path conversion (`<rect>` / `<ellipse>` etc.)
- Path data normalization and compression
- Color minification (`#ffffff` → `#fff`, named colors)
- Default attribute removal, useless defs/stroke/fill removal
- Group flattening where safe

### JSON (via serde_json)
- Whitespace removal
- Key ordering (optional)
- UTF-8 optimization

## Examples

See the `examples/` directory for sample files:

```bash
# JavaScript minification
cargo run -- examples/sample.js --stats

# CSS minification
cargo run -- examples/sample.css --stats

# JSON minification
cargo run -- examples/sample.json --stats
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Format code
cargo fmt

# Lint
cargo clippy
```

## Safety and Correctness

mni prioritizes **correctness over compression**. We've disabled certain aggressive SWC optimizations that can produce invalid JavaScript in edge cases:

- `collapse_vars` - Can create invalid left-hand assignments
- `inline` - Aggressive inlining can break code semantics
- `sequences` - Comma sequence optimization can produce invalid syntax

**Result**: All output is guaranteed to be valid JavaScript that re-parses correctly.

See [BUGS.md](./BUGS.md) for detailed information about known issues and fixes.

## Roadmap

Completed:
- JavaScript minification (SWC)
- CSS minification (LightningCSS)
- HTML minification (minify-html, with inline CSS/JS)
- SVG minification (oxvg `safe` preset)
- JSON minification
- CLI interface
- Auto-format detection
- Safe compression (correctness first)
- Source map generation (JS via SWC, CSS via LightningCSS)
- Batch file processing with `--outdir`
- Parallel processing via rayon (disable with `--no-parallel`)
- Per-file compression statistics comparison (`--stats` in batch mode)
- Config file support (`.minirc.json` / `mni.config.json` / `--config`)
- Watch mode (`--watch`) via `notify`
- Comprehensive test suite

Planned:
- Integration with build tools (Vite/webpack/esbuild plugins)

## Architecture

mni is designed as a **unified orchestrator** around best-in-class libraries:

```
┌─────────────────────────────────────────┐
│              mni CLI                    │
│  (Unified interface & orchestration)    │
└─────────────────────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐    ┌───▼────┐   ┌───▼──────┐
│  SWC  │    │Lightning│   │serde_json│
│  JS   │    │CSS CSS  │   │   JSON   │
└───────┘    └─────────┘   └──────────┘
```

This approach:
- Leverages production-proven libraries
- Provides consistent API across formats
- Enables future extensibility
- Maintains high performance

## Why mni?

1. **Production-Ready**: Built on libraries powering the world's largest applications
2. **Unified Interface**: One tool for all your minification needs
3. **Optimal Performance**: Best-in-class speed for each format
4. **Modern Rust**: Memory-safe, fast, and reliable
5. **Battle-Tested**: Based on SWC (Next.js, Deno) and LightningCSS (Parcel)

## License

MIT

## Contributing

Contributions welcome! Please read our contributing guidelines and submit PRs.

## Acknowledgments

Built with:
- [SWC](https://swc.rs/) - Speedy Web Compiler
- [LightningCSS](https://lightningcss.dev/) - Fast CSS parser and transformer
- [serde_json](https://github.com/serde-rs/json) - JSON serialization

## Support

- [GitHub Issues](https://github.com/epistates/mni/issues)
- [Documentation](https://github.com/epistates/mni)
