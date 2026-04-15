# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-04-15

### Added

- **HTML minification** via `minify-html` 0.16, including inline `<style>`
  (via LightningCSS) and inline `<script>` (via minify-js). New
  `Minifier::minify_html` method and `.html` / `.htm` extension detection.
- **SVG minification** via `oxvg_optimiser` 0.0.5 using the correctness-first
  `Jobs::safe()` preset. Real round-trip XML parseability is covered by the
  integration tests. New `Minifier::minify_svg` method and `.svg` detection
  (both by extension and by `<svg` content sniff).
- **Batch mode** (`--outdir <DIR>`): accept multiple positional inputs and
  write each minified file to the output directory. Required when more than
  one input is given. Per-file errors are reported individually and surface a
  non-zero exit without aborting the batch.
- **Parallel processing** via rayon across batch inputs. Disable with
  `--no-parallel` for deterministic sequential runs. The existing
  `MinifyOptions.parallel` flag is now honored — it was a no-op in 0.1.x.
- **Watch mode** (`--watch`) via `notify` 8.2. Performs an initial build,
  then re-runs on filesystem events with a 150ms debounce to coalesce editor
  save bursts. Rebuild errors are logged but do not stop the watcher.
- **Config file support**: auto-discovers `.minirc.json` or
  `mni.config.json` in the current directory, or loads an explicit path via
  `--config <FILE>`. Files are partial JSON overlays of `MinifyOptions`.
  Precedence: built-in defaults → preset → config file → explicit CLI flags.
  Skip auto-discovery with `--no-config`.
- **Per-file stats table** in batch mode when `--stats` is set. Aligned
  columns for file name, original size, minified size, reduction %, and
  time, followed by aggregate totals.
- **`#sourceMappingURL` footer** appended to minified JS/CSS output when
  `--source-map` is used with a real output file, pointing at the sibling
  `.map` file. CSS uses the `/*# ... */` form; JS uses `//# ...`.
- New library convenience methods `Minifier::minify_js_with_name` and
  `Minifier::minify_css_with_name` that thread a filename into the source
  map `sources` array.

### Changed

- CLI positional input changed from `Option<PathBuf>` to `Vec<PathBuf>` to
  support batch mode. Single-input and stdin flows are preserved unchanged.
- CLI flag parsing now uses `clap::ArgMatches::value_source` to distinguish
  explicit flags from `default_value` fallbacks. This is what lets config
  files set e.g. `mangle: false` without clap silently clobbering it back to
  `true` from its default.
- Internal `minify::js::minify` and `minify::css::minify` now take an
  `Option<&str>` filename so source maps reference the real input path
  instead of `<anon>`.
- `lightningcss` pinned to `=1.0.0-alpha.63` (down from `alpha.71`) for
  compatibility with `oxvg_ast 0.0.5`, which requires a `grid` feature that
  was removed in later alphas. The `PrinterOptions` / source map API is
  identical between the two versions, so no CSS behavior changes. Will be
  bumped in lockstep when oxvg upgrades its lightningcss bound.
- `MinifyOptions.parallel` field documentation updated: it now controls
  batch parallelism via `--no-parallel` instead of being a no-op.

### Fixed

- **`--source-map` actually generates source maps.** In 0.1.x the CLI flag
  was wired through to `MinifyOptions`, but `minify::js` and `minify::css`
  hard-coded `map: None` with a TODO. JS now uses SWC's
  `build_source_map` + `swc_sourcemap::SourceMap::to_writer`; CSS uses
  `parcel_sourcemap::SourceMap` + `PrinterOptions.source_map`. Both respect
  `MinifyOptions.sources_content`.
- The `development` preset sets `source_map: true`, which previously had no
  observable effect. It now actually produces a source map.
- Library docstring claims around "Full source map support" and "Parallel:
  Multi-threaded processing with rayon" are now accurate. The latter claim
  was removed; the former is implemented.
- Extension-based format detection for `.html`/`.htm`/`.svg`, plus content
  sniffing for HTML (`<!doctype html>` / `<html`) and SVG (`<svg`), so
  `minify_auto` routes these formats correctly.

### Dependencies

- **Added**: `minify-html = "0.16"`, `oxvg_optimiser = "0.0.5"`,
  `oxvg_ast = { version = "0.0.5", features = ["roxmltree"] }`,
  `notify = "8.2"`, `parcel_sourcemap = "2.1"`.
- **Added (dev)**: `roxmltree = "0.20"` for SVG round-trip verification in
  integration tests.
- **Feature added**: `swc_core` now uses the `common_sourcemap` feature so
  `swc_common::SourceMap::build_source_map` is available.
- **Pinned**: `lightningcss = "=1.0.0-alpha.63"` (see Changed section).

## [0.1.1] - 2026-04-09

### Fixed

- `conditionals:true` optimization producing invalid `&&` / `||` assignments (#1).
  `if (cond) x = y;` was minified to `cond&&x=y;` (invalid JS) instead of
  `cond&&(x=y);`. Root cause: missing SWC `fixer` pass after `optimize()`.

### Changed

- Re-enabled `conditionals`, `bools`, and `sequences` compress optimizations that
  were previously disabled as workarounds, improving compression ratios.
- Updated dependencies to latest versions:
  - `swc_core` 47.0 -> 63.1
  - `lightningcss` 1.0.0-alpha.68 -> 1.0.0-alpha.71
  - `clap` 4.5 -> 4.6
  - `criterion` 0.5 -> 0.8
  - `insta` 1.40 -> 1.47
  - `sourcemap` 9.0 -> 9.3

## [0.1.0] - 2026-04-09

### Added

- Initial release.
- JavaScript minification via SWC (ES5-ESNext, TypeScript).
- CSS minification via LightningCSS.
- JSON minification via serde_json.
- CLI with presets (dev, prod, aggressive).
- Source map support.
- Parallel processing with rayon.
