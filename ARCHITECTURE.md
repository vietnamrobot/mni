# mni Architecture

## Design Philosophy

**mni** is built on a pragmatic architectural principle: **orchestrate the best, don't reinvent the wheel**.

Instead of building parsers, ASTs, and optimizers from scratch, mni acts as a **unified interface** around battle-tested, production-proven libraries used by millions of developers worldwide.

## Core Components

### 1. JavaScript Minification - SWC

**Library**: `swc_core` v47+
**Production Usage**: Next.js, Deno, Vercel, Parcel
**Performance**: 7x faster than Terser, 20x faster than Babel

**What we get**:
- Arena-allocated AST (via SWC's internal allocator)
- Production-tested parser supporting ES5 through ESNext
- Advanced minifier with:
  - Identifier mangling with scope analysis
  - Dead code elimination
  - Constant folding
  - Boolean optimization
  - Arrow function optimization
  - Template literal optimization

**Integration approach**:
```rust
swc_core = { version = "47.0", features = [
    "ecma_ast",
    "ecma_minifier",
    "ecma_parser",
    "ecma_transforms",
    "ecma_codegen",
    "ecma_visit",
    "common"
] }
```

### 2. CSS Minification - LightningCSS

**Library**: `lightningcss` v1.0.0-alpha.68
**Production Usage**: Parcel bundler
**Performance**: 100x faster than cssnano, 2.7M+ lines/sec

**What we get**:
- Rust-native CSS parser
- Comprehensive minification:
  - Whitespace removal
  - Color optimization (#ffffff → #fff)
  - Length optimization (0px → 0)
  - Property merging
  - Vendor prefix optimization
  - Calc() simplification
- Source map support (ready to integrate)

**Why not write our own CSS minifier?**
CSS has complex edge cases (vendor prefixes, browser compatibility, cascade rules). LightningCSS handles all of this and is actively maintained by Parcel team.

### 3. JSON Minification - serde_json

**Library**: `serde_json` v1.0
**Production Usage**: Rust ecosystem standard
**Performance**: Native Rust serialization speed

**What we get**:
- Proven JSON parser
- Standards-compliant
- Optimal performance
- Zero edge cases

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│                    mni CLI/Library                    │
│  ┌────────────────────────────────────────────────┐  │
│  │           Unified Interface Layer              │  │
│  │  - Config management                           │  │
│  │  - Format detection                            │  │
│  │  - Option mapping                              │  │
│  │  - Statistics collection                       │  │
│  └────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    ┌────▼────┐    ┌────▼─────┐   ┌────▼──────┐
    │   SWC   │    │Lightning │   │serde_json │
    │  Rust   │    │   CSS    │   │   Rust    │
    │ v47.0   │    │ Rust     │   │ v1.0      │
    └─────────┘    │ v1.0-α68 │   └───────────┘
    Powers:        └──────────┘   Powers:
    Next.js        Powers:         Rust ecosystem
    Deno           Parcel
    Vercel
```

## Module Structure

```
mni/
├── src/
│   ├── lib.rs              # Public API, MinifyResult, Minifier
│   ├── main.rs             # CLI implementation (clap)
│   ├── config.rs           # MinifyOptions, Target, Presets
│   └── minify/
│       ├── mod.rs          # Module exports
│       ├── js.rs           # SWC integration
│       ├── css.rs          # LightningCSS integration
│       └── json.rs         # serde_json integration
├── tests/
│   └── integration_test.rs # Comprehensive integration tests
├── examples/
│   ├── sample.js           # Example JS file
│   ├── sample.css          # Example CSS file
│   └── sample.json         # Example JSON file
└── benches/
    └── minify_bench.rs     # Performance benchmarks
```

## Key Design Decisions

### 1. Why Not Build Our Own Parser?

**Parsers are hard**. They require:
- Handling all ECMAScript versions (ES5 → ESNext)
- Supporting JSX, TypeScript, decorators
- Maintaining compatibility with evolving specs
- Edge case handling (thousands of test cases)
- Performance optimization (arena allocation, etc.)

SWC has **years of production testing** and is maintained by a dedicated team.

**ROI**: Building a competitive parser would take 6-12 months. Using SWC: 1 day.

### 2. Why Orchestration Architecture?

**Benefits**:
- Leverage battle-tested code (fewer bugs)
- Get free updates as libraries improve
- Focus on user experience, not parser internals
- Significantly faster development
- Production-ready from day one

**Trade-offs**:
- Dependency on external libraries
- Version compatibility (managed via features)
- Less control over specific optimizations

**Decision**: The trade-offs are overwhelmingly worth it. Users get production-proven code immediately.

### 3. Configuration Mapping

Each library has its own configuration. mni provides:
- **Unified config** (`MinifyOptions`)
- **Presets** (dev, prod, aggressive)
- **Sensible defaults**

Example mapping:
```rust
// User config
MinifyOptions {
    mangle: true,
    compress: true,
    drop_console: true,
    ...
}

// Maps to SWC
SwcMinifyOptions {
    compress: Some(CompressOptions {
        drop_console: true,
        dead_code: true,
        ...
    }),
    mangle: Some(MangleOptions {
        ...
    }),
}

// And to LightningCSS
LightningMinifyOptions::default()
```

### 4. Statistics Collection

We wrap each library call to collect:
- Original size
- Minified size
- Compression ratio
- Processing time

This provides consistent metrics across all formats.

## Performance Characteristics

### JavaScript (SWC)
- **Parse**: O(n) with arena allocation
- **Optimize**: O(n×p) where p = passes
- **Codegen**: O(n)
- **Total**: ~18ms for 1.3KB (dev machine)

### CSS (LightningCSS)
- **Parse**: O(n)
- **Optimize**: O(n)
- **Codegen**: O(n)
- **Total**: ~6ms for 1.7KB

### JSON (serde_json)
- **Parse**: O(n)
- **Serialize**: O(n)
- **Total**: <1ms for 671B

## Future Extensions

### Source Maps (Next Priority)
- SWC: Built-in source map support
- LightningCSS: Has source map features
- Integration: Map both to common format

### Batch Processing
```rust
pub fn minify_batch(&self, files: &[PathBuf]) -> Result<Vec<MinifyResult>> {
    files.par_iter()  // rayon parallel iterator
        .map(|f| self.minify_file(f))
        .collect()
}
```

### Watch Mode
```rust
use notify::Watcher;
// Watch for file changes and auto-minify
```

### Config File (.minirc)
```toml
[javascript]
target = "es2020"
mangle = true
compress = true

[css]
minify = true

[json]
pretty = false
```

## Testing Strategy

1. **Integration Tests**: Verify each format works end-to-end
2. **Regression Tests**: Snapshot testing with `insta` crate
3. **Benchmark Tests**: Track performance over time with `criterion`
4. **Real-World Tests**: Test against actual projects (React, Vue, etc.)

## Benchmarking

```bash
cargo bench
```

Compares against:
- Terser (via Node.js)
- esbuild (via CLI)
- cssnano (via PostCSS)

## Error Handling

- Parse errors: Bubble up from underlying libraries with context
- Config errors: Validate at creation time
- IO errors: Comprehensive error messages with file paths

## Dependencies Philosophy

**Principle**: Only depend on libraries that are:
1. Production-proven (millions of users)
2. Actively maintained (commits in last 3 months)
3. Well-documented
4. Performance-critical

**Current dependencies**: SWC, LightningCSS, serde_json all meet this bar.

## Summary

mni is a **smart orchestrator**, not a from-scratch implementation. This architectural choice delivers:

- Production-ready code (day 1)
- World-class performance (7-100x faster than alternatives)
- Comprehensive features (years of development)
- Future-proof (actively maintained libraries)
- Minimal maintenance (upstream handles edge cases)

