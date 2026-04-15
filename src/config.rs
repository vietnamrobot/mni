//! Configuration options for minification.
//!
//! This module provides the configuration types for controlling minification behavior,
//! including target ECMAScript versions, compression options, and preset configurations.

use serde::{Deserialize, Serialize};

/// Target JavaScript version for minification.
///
/// Determines which ECMAScript features can be used in the output.
/// Lower versions produce more compatible code but may be larger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Target {
    /// ECMAScript 5 (IE9+) - Maximum compatibility
    ES5,
    /// ECMAScript 2015 (ES6) - Arrow functions, classes, let/const
    ES2015,
    /// ECMAScript 2016 - Exponentiation operator, Array.includes
    ES2016,
    /// ECMAScript 2017 - Async/await, Object entries/values
    ES2017,
    /// ECMAScript 2018 - Rest/spread properties, async iteration
    ES2018,
    /// ECMAScript 2019 - Array.flat, Object.fromEntries
    ES2019,
    /// ECMAScript 2020 - Optional chaining, nullish coalescing (default)
    #[default]
    ES2020,
    /// ECMAScript 2021 - Logical assignment, numeric separators
    ES2021,
    /// ECMAScript 2022 - Top-level await, class fields
    ES2022,
    /// ECMAScript 2023 - Array findLast, hashbang grammar
    ES2023,
    /// ECMAScript 2024 - Latest stable features
    ES2024,
    /// Latest ECMAScript features including proposals
    ESNext,
}

/// Main minification configuration options.
///
/// Controls all aspects of the minification process including compression,
/// mangling, source maps, and target ECMAScript version.
///
/// # Examples
///
/// ```rust
/// use mni::{MinifyOptions, Target};
///
/// // Use default settings (recommended for most cases)
/// let options = MinifyOptions::default();
///
/// // Or use presets
/// let prod_options = MinifyOptions::production();
/// let dev_options = MinifyOptions::development();
/// let aggressive_options = MinifyOptions::aggressive();
///
/// // Or customize completely
/// let custom = MinifyOptions {
///     target: Target::ES2015,
///     mangle: true,
///     compress: true,
///     source_map: true,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct MinifyOptions {
    /// Target ECMAScript version (default: ES2020)
    pub target: Target,

    /// Enable identifier mangling to shorten variable names (default: true)
    pub mangle: bool,

    /// Mangle property names - UNSAFE, requires careful configuration (default: false)
    pub mangle_props: bool,

    /// Reserved identifiers that should never be mangled (e.g., external API names)
    pub reserved: Vec<String>,

    /// Enable compression optimizations like dead code elimination (default: true)
    pub compress: bool,

    /// Fine-grained compression options
    pub compress_options: CompressOptions,

    /// Generate source maps for debugging (default: false)
    pub source_map: bool,

    /// Include original source content in the source map (default: true)
    pub sources_content: bool,

    /// Preserve class names for debugging/reflection (default: false)
    pub keep_classnames: bool,

    /// Preserve function names for debugging/profiling (default: false)
    pub keep_fnames: bool,

    /// Allow mangling of top-level scope identifiers (default: false)
    pub toplevel: bool,

    /// Reserved for future parallel multi-file processing. Currently a no-op —
    /// setting this flag has no effect until batch minification lands.
    pub parallel: bool,
}

impl Default for MinifyOptions {
    fn default() -> Self {
        Self {
            target: Target::default(),
            mangle: true,
            mangle_props: false,
            reserved: Vec::new(),
            compress: true,
            compress_options: CompressOptions::default(),
            source_map: false,
            sources_content: true,
            keep_classnames: false,
            keep_fnames: false,
            toplevel: false,
            parallel: true,
        }
    }
}

/// Fine-grained compression and optimization options.
///
/// Controls specific JavaScript optimizations performed during minification.
/// Some aggressive options are disabled by default for safety.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct CompressOptions {
    /// Remove unreachable code (default: true)
    pub dead_code: bool,

    /// Optimize constant expressions like `1 + 2` to `3` (default: true)
    pub evaluate: bool,

    /// Join consecutive var statements (default: true)
    pub join_vars: bool,

    /// Optimize loop conditions and bodies (default: true)
    pub loops: bool,

    /// Remove unused variables and functions (default: true)
    pub unused: bool,

    /// Collapse single-use variables - DISABLED by default due to SWC bugs (default: false)
    pub collapse_vars: bool,

    /// Optimize if-else and ternary conditionals (default: true)
    pub conditionals: bool,

    /// Optimize comparison expressions (default: true)
    pub comparisons: bool,

    /// Optimize boolean expressions and conditions (default: true)
    pub booleans: bool,

    /// Remove all console.* statements (default: false)
    pub drop_console: bool,

    /// Remove debugger statements (default: true)
    pub drop_debugger: bool,

    /// Hoist function declarations to top of scope (default: false)
    pub hoist_funs: bool,

    /// Hoist var declarations to top of scope (default: false)
    pub hoist_vars: bool,

    /// Number of optimization passes (higher = more compression, slower) (default: 1)
    pub passes: usize,

    /// Enable all unsafe transformations (default: false)
    pub unsafe_opt: bool,

    /// Unsafe arrow function optimizations (default: false)
    pub unsafe_arrows: bool,

    /// Unsafe math optimizations that may lose precision (default: false)
    pub unsafe_math: bool,

    /// Unsafe method optimizations (default: false)
    pub unsafe_methods: bool,

    /// Unsafe proto optimizations (default: false)
    pub unsafe_proto: bool,

    /// Unsafe regex optimizations (default: false)
    pub unsafe_regexp: bool,

    /// Function inlining level - DISABLED by default due to SWC bugs (default: None)
    pub inline: InlineLevel,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            dead_code: true,
            evaluate: true,
            join_vars: true,
            loops: true,
            unused: true,
            collapse_vars: false, // Disabled: causes SWC to generate invalid JS in some cases
            conditionals: true,
            comparisons: true,
            booleans: true,
            drop_console: false,
            drop_debugger: true,
            hoist_funs: false,
            hoist_vars: false,
            passes: 1,
            unsafe_opt: false,
            unsafe_arrows: false,
            unsafe_math: false,
            unsafe_methods: false,
            unsafe_proto: false,
            unsafe_regexp: false,
            inline: InlineLevel::None, // Disabled: can cause SWC bugs with aggressive inlining
        }
    }
}

/// Function inlining level for compression.
///
/// Controls how aggressively the minifier inlines function calls.
/// Higher levels provide more compression but increase processing time and may cause bugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineLevel {
    /// No inlining (safest, recommended for most use cases)
    None,
    /// Inline simple, single-statement functions
    Simple,
    /// Aggressive inlining of all functions (may cause SWC bugs, use with caution)
    Aggressive,
}

impl MinifyOptions {
    /// Creates production-optimized settings.
    ///
    /// Balanced configuration for production builds with good compression and reliability.
    ///
    /// Settings:
    /// - Mangling: enabled
    /// - Compression: enabled (2 passes)
    /// - Source maps: disabled
    /// - Safe optimizations only
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::MinifyOptions;
    ///
    /// let options = MinifyOptions::production();
    /// assert!(options.mangle);
    /// assert!(options.compress);
    /// assert_eq!(options.compress_options.passes, 2);
    /// ```
    #[must_use]
    pub fn production() -> Self {
        Self {
            mangle: true,
            compress: true,
            compress_options: CompressOptions {
                passes: 2,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Creates development-optimized settings.
    ///
    /// Fast minification with source maps for debugging.
    ///
    /// Settings:
    /// - Mangling: disabled (readable names)
    /// - Compression: disabled (fast builds)
    /// - Source maps: enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::MinifyOptions;
    ///
    /// let options = MinifyOptions::development();
    /// assert!(!options.mangle);
    /// assert!(!options.compress);
    /// assert!(options.source_map);
    /// ```
    #[must_use]
    pub fn development() -> Self {
        Self {
            mangle: false,
            compress: false,
            source_map: true,
            ..Default::default()
        }
    }

    /// Creates settings for maximum compression.
    ///
    /// Aggressive optimization for smallest possible output.
    /// WARNING: Uses potentially unsafe transformations that may cause bugs.
    /// Test thoroughly when using this preset.
    ///
    /// Settings:
    /// - Mangling: enabled (including top-level)
    /// - Compression: enabled (3 passes)
    /// - Aggressive inlining and variable collapsing
    /// - All safe and some unsafe optimizations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mni::MinifyOptions;
    ///
    /// let options = MinifyOptions::aggressive();
    /// assert!(options.mangle);
    /// assert!(options.toplevel);
    /// assert_eq!(options.compress_options.passes, 3);
    /// ```
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            mangle: true,
            compress: true,
            compress_options: CompressOptions {
                passes: 3,
                inline: InlineLevel::Aggressive,
                collapse_vars: true,
                unused: true,
                dead_code: true,
                ..Default::default()
            },
            toplevel: true,
            ..Default::default()
        }
    }
}
