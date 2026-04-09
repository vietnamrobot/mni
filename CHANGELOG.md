# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
