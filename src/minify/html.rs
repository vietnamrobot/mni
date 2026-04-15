//! HTML minification using `minify-html`

use crate::config::MinifyOptions;
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use minify_html::{Cfg, minify as minify_html};
use std::time::Instant;

/// Minify HTML code using `minify-html`.
///
/// Inline `<style>` blocks are minified via `LightningCSS` and inline `<script>`
/// blocks via `minify-js` (both bundled into `minify-html`). The `MinifyOptions`
/// passed in currently only controls whether inline CSS/JS are minified.
pub fn minify(source: &str, _options: &MinifyOptions) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    let cfg = Cfg {
        minify_css: true,
        minify_js: true,
        minify_doctype: true,
        ..Cfg::default()
    };

    let minified_bytes = minify_html(source.as_bytes(), &cfg);
    let minified_code =
        String::from_utf8(minified_bytes).context("minify-html produced non-UTF-8 output")?;

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
