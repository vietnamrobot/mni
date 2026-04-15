//! JavaScript minification using SWC

use crate::config::{MinifyOptions, Target};
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Instant;
use swc_core::{
    common::{
        BytePos, FileName, GLOBALS, Globals, LineCol, Mark, SourceMap,
        source_map::SourceMapGenConfig, sync::Lrc,
    },
    ecma::{
        ast::EsVersion,
        codegen::{Emitter, text_writer::JsWriter},
        minifier::option::{
            CompressOptions, ExtraOptions, MangleOptions, MinifyOptions as SwcMinifyOptions,
        },
        parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer},
        transforms::base::{fixer::fixer, resolver},
        visit::VisitMutWith,
    },
};

struct SrcMapConfig {
    inline_sources_content: bool,
}

impl SourceMapGenConfig for SrcMapConfig {
    fn file_name_to_source(&self, f: &FileName) -> String {
        match f {
            FileName::Real(p) => p.to_string_lossy().into_owned(),
            _ => f.to_string(),
        }
    }

    fn inline_sources_content(&self, _f: &FileName) -> bool {
        self.inline_sources_content
    }
}

fn file_name_for(filename: Option<&str>) -> FileName {
    filename.map_or(FileName::Anon, |name| FileName::Real(PathBuf::from(name)))
}

fn build_swc_minify_options(options: &MinifyOptions) -> SwcMinifyOptions {
    SwcMinifyOptions {
        compress: if options.compress {
            Some(CompressOptions {
                // Explicitly configure all options to avoid SWC bugs
                dead_code: options.compress_options.dead_code,
                drop_console: options.compress_options.drop_console,
                drop_debugger: options.compress_options.drop_debugger,
                evaluate: options.compress_options.evaluate,
                join_vars: options.compress_options.join_vars,
                loops: options.compress_options.loops,
                unused: options.compress_options.unused,
                conditionals: options.compress_options.conditionals,
                comparisons: options.compress_options.comparisons,
                bools: options.compress_options.booleans,
                hoist_fns: options.compress_options.hoist_funs,
                hoist_vars: options.compress_options.hoist_vars,

                // Disable aggressive optimizations known to produce invalid
                // code even after the fixer pass
                inline: 0,
                collapse_vars: false,

                ..Default::default()
            })
        } else {
            None
        },
        mangle: if options.mangle {
            Some(MangleOptions {
                top_level: Some(options.toplevel),
                keep_class_names: options.keep_classnames,
                keep_fn_names: options.keep_fnames,
                reserved: options.reserved.iter().map(|s| s.as_str().into()).collect(),
                ..Default::default()
            })
        } else {
            None
        },
        ..Default::default()
    }
}

fn serialize_source_map(
    cm: &Lrc<SourceMap>,
    mappings: &[(BytePos, LineCol)],
    inline_sources_content: bool,
) -> Result<String> {
    let srcmap = cm.build_source_map(
        mappings,
        None,
        SrcMapConfig {
            inline_sources_content,
        },
    );
    let mut out = vec![];
    srcmap
        .to_writer(&mut out)
        .context("Failed to serialize source map")?;
    String::from_utf8(out).context("Source map was not valid UTF-8")
}

/// Minify JavaScript code using SWC
pub fn minify(
    source: &str,
    filename: Option<&str>,
    options: &MinifyOptions,
) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(Lrc::new(file_name_for(filename)), source.to_string());

    // Parse with SWC
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: true,
            ..Default::default()
        }),
        target_to_es_version(options.target),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let mut program = parser
        .parse_program()
        .map_err(|e| anyhow::anyhow!("Failed to parse JavaScript: {}", e.kind().msg()))?;

    // Apply SWC minification
    program = GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        // Resolve identifiers
        program.visit_mut_with(&mut resolver(unresolved_mark, top_level_mark, false));

        // Minify
        if options.compress || options.mangle {
            let minify_options = build_swc_minify_options(options);

            let mut program = swc_core::ecma::minifier::optimize(
                program,
                cm.clone(),
                None,
                None,
                &minify_options,
                &ExtraOptions {
                    unresolved_mark,
                    top_level_mark,
                    mangle_name_cache: Option::default(),
                },
            );

            // fixer corrects parenthesization after minifier transforms
            program.visit_mut_with(&mut fixer(None));
            program
        } else {
            program
        }
    });

    // Generate minified code (and optionally collect source map mappings)
    let mut buf = vec![];
    let mut srcmap_buf: Vec<(BytePos, LineCol)> = vec![];
    {
        let srcmap_slot = if options.source_map {
            Some(&mut srcmap_buf)
        } else {
            None
        };
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, srcmap_slot);
        let mut emitter = Emitter {
            cfg: swc_core::ecma::codegen::Config::default().with_minify(true),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };

        emitter
            .emit_program(&program)
            .context("Failed to emit code")?;
    }

    let minified_code = String::from_utf8(buf).context("Failed to convert output to UTF-8")?;

    let map = if options.source_map {
        Some(serialize_source_map(
            &cm,
            &srcmap_buf,
            options.sources_content,
        )?)
    } else {
        None
    };

    let minified_size = minified_code.len();
    let elapsed = start.elapsed();

    let stats =
        MinifyStats::with_sizes(original_size, minified_size).with_time(elapsed.as_millis());

    Ok(MinifyResult {
        code: minified_code,
        map,
        stats,
    })
}

/// Convert target to SWC `EsVersion`
const fn target_to_es_version(target: Target) -> EsVersion {
    match target {
        Target::ES5 => EsVersion::Es5,
        Target::ES2015 => EsVersion::Es2015,
        Target::ES2016 => EsVersion::Es2016,
        Target::ES2017 => EsVersion::Es2017,
        Target::ES2018 => EsVersion::Es2018,
        Target::ES2019 => EsVersion::Es2019,
        Target::ES2020 => EsVersion::Es2020,
        Target::ES2021 => EsVersion::Es2021,
        Target::ES2022 => EsVersion::Es2022,
        Target::ES2023 | Target::ES2024 | Target::ESNext => EsVersion::EsNext,
    }
}
