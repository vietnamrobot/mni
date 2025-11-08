//! # mni - A World-Class Minifier
//!
//! A high-performance minifier for JavaScript, CSS, and JSON written in Rust.
//!
//! Built on industry-leading libraries:
//! - **JavaScript**: SWC (powers Next.js, Deno, Vercel)
//! - **CSS**: `LightningCSS` (100x faster than cssnano, powers Parcel)
//! - **JSON**: `serde_json` (Rust standard)
//!
//! ## Features
//!
//! - **Production-Ready**: Built on battle-tested libraries
//! - **Blazing Fast**: SWC + `LightningCSS` performance
//! - **High Compression**: Terser-level minification quality
//! - **Multi-format**: JavaScript (ES5-ESNext), CSS, JSON
//! - **Source Maps**: Full source map support
//! - **Parallel**: Multi-threaded processing with rayon
//!
//! ## Example
//!
//! ```rust,no_run
//! use mni::{Minifier, MinifyOptions, Target};
//!
//! let code = r#"
//!     function hello(name) {
//!         console.log("Hello, " + name + "!");
//!     }
//! "#;
//!
//! let options = MinifyOptions {
//!     target: Target::ES2015,
//!     mangle: true,
//!     compress: true,
//!     ..Default::default()
//! };
//!
//! let minified = Minifier::new(options).minify_js(code).unwrap();
//! println!("{}", minified.code);
//! ```

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

pub mod config;
pub mod minify;

use anyhow::Result;
pub use config::{MinifyOptions, Target};

/// Main minifier interface that provides methods to minify JavaScript, CSS, and JSON.
///
/// Create a new instance with `Minifier::new()` and configure it with `MinifyOptions`.
///
/// # Examples
///
/// ```rust
/// use mni::{Minifier, MinifyOptions};
///
/// let minifier = Minifier::new(MinifyOptions::default());
/// let result = minifier.minify_js("function test() { return 42; }").unwrap();
/// println!("{}", result.code);
/// ```
pub struct Minifier {
    options: MinifyOptions,
}

/// Result of a minification operation containing the minified code, optional source map, and statistics.
pub struct MinifyResult {
    /// Minified code output
    pub code: String,
    /// Source map (if source map generation is enabled in options)
    pub map: Option<String>,
    /// Statistics about the minification operation (sizes, compression ratio, time)
    pub stats: MinifyStats,
}

/// Statistics collected during the minification process.
///
/// Provides metrics about the size reduction and performance of the minification operation.
#[derive(Debug, Default, Clone)]
pub struct MinifyStats {
    /// Original input size in bytes
    pub original_size: usize,
    /// Minified output size in bytes
    pub minified_size: usize,
    /// Compression ratio as a value from 0.0 to 1.0 (e.g., 0.42 means 42% reduction)
    pub compression_ratio: f64,
    /// Time taken to minify in milliseconds
    pub time_ms: u128,
}

impl Minifier {
    /// Creates a new minifier with the given options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::{Minifier, MinifyOptions, Target};
    ///
    /// let options = MinifyOptions {
    ///     target: Target::ES2020,
    ///     mangle: true,
    ///     compress: true,
    ///     ..Default::default()
    /// };
    /// let minifier = Minifier::new(options);
    /// ```
    #[must_use]
    pub const fn new(options: MinifyOptions) -> Self {
        Self { options }
    }

    /// Minifies JavaScript code using SWC.
    ///
    /// Supports ES5 through `ESNext` syntax including TypeScript decorators.
    ///
    /// # Errors
    ///
    /// Returns an error if the JavaScript code cannot be parsed or if minification fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::{Minifier, MinifyOptions};
    ///
    /// let minifier = Minifier::new(MinifyOptions::default());
    /// let result = minifier.minify_js("const x = 1 + 2;").unwrap();
    /// assert!(result.code.len() < 16);
    /// ```
    pub fn minify_js(&self, source: &str) -> Result<MinifyResult> {
        minify::js::minify(source, &self.options)
    }

    /// Minifies CSS code using `LightningCSS`.
    ///
    /// Performs optimizations like whitespace removal, color minification, and property merging.
    ///
    /// # Errors
    ///
    /// Returns an error if the CSS code cannot be parsed or if minification fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::{Minifier, MinifyOptions};
    ///
    /// let minifier = Minifier::new(MinifyOptions::default());
    /// let result = minifier.minify_css(".class { color: #ffffff; }").unwrap();
    /// assert!(result.code.contains("#fff"));
    /// ```
    pub fn minify_css(&self, source: &str) -> Result<MinifyResult> {
        minify::css::minify(source, &self.options)
    }

    /// Minifies JSON code using `serde_json`.
    ///
    /// Removes all unnecessary whitespace while preserving valid JSON structure.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::{Minifier, MinifyOptions};
    ///
    /// let minifier = Minifier::new(MinifyOptions::default());
    /// let result = minifier.minify_json(r#"{ "key": "value" }"#).unwrap();
    /// assert_eq!(result.code, r#"{"key":"value"}"#);
    /// ```
    pub fn minify_json(&self, source: &str) -> Result<MinifyResult> {
        minify::json::minify(source, &self.options)
    }

    /// Auto-detects the format from filename extension or content and minifies accordingly.
    ///
    /// Detection priority:
    /// 1. File extension (.js, .css, .json)
    /// 2. Content analysis (tries JSON parse for objects/arrays)
    /// 3. Defaults to JavaScript
    ///
    /// # Errors
    ///
    /// Returns an error if format detection or minification fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::{Minifier, MinifyOptions};
    ///
    /// let minifier = Minifier::new(MinifyOptions::default());
    /// let result = minifier.minify_auto("const x = 1;", Some("app.js")).unwrap();
    /// assert!(result.stats.minified_size < result.stats.original_size);
    /// ```
    pub fn minify_auto(&self, source: &str, filename: Option<&str>) -> Result<MinifyResult> {
        let format = detect_format(source, filename);
        match format {
            Format::JavaScript => self.minify_js(source),
            Format::CSS => self.minify_css(source),
            Format::JSON => self.minify_json(source),
        }
    }
}

/// Detected file format
#[allow(clippy::upper_case_acronyms)]
enum Format {
    JavaScript,
    CSS,
    JSON,
}

/// Detect format from content and filename
fn detect_format(source: &str, filename: Option<&str>) -> Format {
    // Try filename extension first
    if let Some(name) = filename
        && let Some(ext) = std::path::Path::new(name).extension() {
        if ext.eq_ignore_ascii_case("css") {
            return Format::CSS;
        }
        if ext.eq_ignore_ascii_case("json") {
            return Format::JSON;
        }
        if ext.eq_ignore_ascii_case("js") || ext.eq_ignore_ascii_case("mjs") || ext.eq_ignore_ascii_case("cjs") {
            return Format::JavaScript;
        }
    }

    // Fallback to content analysis
    let trimmed = source.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        // Could be JSON or JS object/array - try JSON parse
        if serde_json::from_str::<serde_json::Value>(source).is_ok() {
            return Format::JSON;
        }
    }

    // Default to JavaScript
    Format::JavaScript
}

impl MinifyStats {
    /// Creates statistics with the given sizes and calculates the compression ratio.
    ///
    /// The compression ratio is calculated as: `1.0 - (minified_size / original_size)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::MinifyStats;
    ///
    /// let stats = MinifyStats::with_sizes(1000, 580);
    /// assert_eq!(stats.original_size, 1000);
    /// assert_eq!(stats.minified_size, 580);
    /// assert!((stats.compression_ratio - 0.42).abs() < 0.01);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn with_sizes(original_size: usize, minified_size: usize) -> Self {
        let compression_ratio = if original_size > 0 {
            1.0 - (minified_size as f64 / original_size as f64)
        } else {
            0.0
        };

        Self {
            original_size,
            minified_size,
            compression_ratio,
            time_ms: 0,
        }
    }

    /// Sets the time taken for the minification operation.
    ///
    /// This method consumes self and returns a new instance with the time set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::MinifyStats;
    ///
    /// let stats = MinifyStats::with_sizes(1000, 580).with_time(42);
    /// assert_eq!(stats.time_ms, 42);
    /// ```
    #[must_use]
    pub const fn with_time(mut self, time_ms: u128) -> Self {
        self.time_ms = time_ms;
        self
    }
}
