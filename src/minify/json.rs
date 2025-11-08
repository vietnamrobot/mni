//! JSON minification

use crate::config::MinifyOptions;
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use std::time::Instant;

/// Minify JSON code
pub fn minify(source: &str, _options: &MinifyOptions) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    // Parse JSON to validate and then re-serialize without whitespace
    let value: serde_json::Value = serde_json::from_str(source).context("Invalid JSON")?;

    let minified = serde_json::to_string(&value).context("Failed to serialize JSON")?;

    let minified_size = minified.len();
    let elapsed = start.elapsed();

    let stats =
        MinifyStats::with_sizes(original_size, minified_size).with_time(elapsed.as_millis());

    Ok(MinifyResult {
        code: minified,
        map: None,
        stats,
    })
}
