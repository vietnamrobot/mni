//! SVG minification using `oxvg` (a Rust port of SVGO).

use crate::config::MinifyOptions;
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use oxvg_ast::{parse::roxmltree::parse, serialize::Node as _, visitor::Info};
use oxvg_optimiser::Jobs;
use std::time::Instant;

/// Minify SVG source using oxvg's correctness-first `safe` preset.
///
/// The `safe` preset maps to SVGO's correctness preset — it skips
/// transformations that can visually change the document (precision loss in
/// path data, id mangling that could break external references, etc.) and
/// sticks to whitespace, comment, metadata, and empty-attribute removal.
pub fn minify(source: &str, _options: &MinifyOptions) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    let minified_code: String = parse(source, |dom, allocator| -> Result<String> {
        let jobs = Jobs::safe();
        jobs.run(dom, &Info::new(allocator))
            .map_err(|e| anyhow::anyhow!("SVG optimisation failed: {e}"))?;
        dom.serialize()
            .map_err(|e| anyhow::anyhow!("SVG serialization failed: {e}"))
    })
    .context("Failed to parse SVG")??;

    let minified_size = minified_code.len();
    let elapsed = start.elapsed();
    let stats =
        MinifyStats::with_sizes(original_size, minified_size).with_time(elapsed.as_millis());

    Ok(MinifyResult {
        code: minified_code,
        map: None,
        stats,
    })
}
