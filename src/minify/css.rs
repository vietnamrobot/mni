//! CSS minification using `LightningCSS`

use crate::config::MinifyOptions;
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use lightningcss::stylesheet::{
    MinifyOptions as LightningMinifyOptions, ParserOptions, PrinterOptions, StyleSheet,
};
use parcel_sourcemap::SourceMap;
use std::time::Instant;

/// Minify CSS code using `LightningCSS`
pub fn minify(
    source: &str,
    filename: Option<&str>,
    options: &MinifyOptions,
) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    let source_name = filename.unwrap_or("input.css");

    // Parse CSS with LightningCSS
    let parser_opts = ParserOptions {
        filename: source_name.to_string(),
        ..ParserOptions::default()
    };
    let mut stylesheet = StyleSheet::parse(source, parser_opts)
        .map_err(|e| anyhow::anyhow!("Failed to parse CSS: {e:?}"))?;

    // Minify
    stylesheet
        .minify(LightningMinifyOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to minify CSS: {e:?}"))?;

    // Set up source map collector when requested
    let mut source_map = if options.source_map {
        let mut sm = SourceMap::new("/");
        let idx = sm.add_source(source_name);
        if options.sources_content {
            sm.set_source_content(idx as usize, source)
                .context("Failed to attach source content to CSS source map")?;
        }
        Some(sm)
    } else {
        None
    };

    // Generate minified output
    let minified_code = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            source_map: source_map.as_mut(),
            ..Default::default()
        })
        .context("Failed to generate minified CSS")?;

    let map = if let Some(mut sm) = source_map {
        Some(
            sm.to_json(None)
                .context("Failed to serialize CSS source map")?,
        )
    } else {
        None
    };

    let minified_size = minified_code.code.len();
    let elapsed = start.elapsed();

    let stats =
        MinifyStats::with_sizes(original_size, minified_size).with_time(elapsed.as_millis());

    Ok(MinifyResult {
        code: minified_code.code,
        map,
        stats,
    })
}
