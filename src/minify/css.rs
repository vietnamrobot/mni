//! CSS minification using `LightningCSS`

use crate::config::MinifyOptions;
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use lightningcss::stylesheet::{
    MinifyOptions as LightningMinifyOptions, ParserOptions, PrinterOptions, StyleSheet,
};
use std::time::Instant;

/// Minify CSS code using `LightningCSS`
pub fn minify(source: &str, _options: &MinifyOptions) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    // Parse CSS with LightningCSS
    let mut stylesheet = StyleSheet::parse(source, ParserOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to parse CSS: {e:?}"))?;

    // Minify
    stylesheet
        .minify(LightningMinifyOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to minify CSS: {e:?}"))?;

    // Generate minified output
    let minified_code = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..Default::default()
        })
        .context("Failed to generate minified CSS")?;

    let minified_size = minified_code.code.len();
    let elapsed = start.elapsed();

    let stats =
        MinifyStats::with_sizes(original_size, minified_size).with_time(elapsed.as_millis());

    Ok(MinifyResult {
        code: minified_code.code,
        map: None, // TODO: Implement source map from LightningCSS
        stats,
    })
}
