//! JavaScript minification using SWC

use crate::config::{MinifyOptions, Target};
use crate::{MinifyResult, MinifyStats};
use anyhow::{Context, Result};
use std::time::Instant;
use swc_core::{
    common::{FileName, GLOBALS, Globals, Mark, SourceMap, sync::Lrc},
    ecma::{
        ast::EsVersion,
        codegen::{Emitter, text_writer::JsWriter},
        minifier::option::{
            CompressOptions, ExtraOptions, MangleOptions, MinifyOptions as SwcMinifyOptions,
        },
        parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer},
        transforms::base::resolver,
        visit::VisitMutWith,
    },
};

/// Minify JavaScript code using SWC
pub fn minify(source: &str, options: &MinifyOptions) -> Result<MinifyResult> {
    let start = Instant::now();
    let original_size = source.len();

    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(Lrc::new(FileName::Anon), source.to_string());

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
            let minify_options = SwcMinifyOptions {
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

                        // Disable aggressive optimizations that can produce invalid code
                        // See: https://github.com/swc-project/swc/issues/...
                        inline: 0, // Disable inlining to prevent invalid AST transformations
                        sequences: 0, // Disable comma sequences that can break assignments
                        collapse_vars: false, // Disable variable collapse (causes the bug)

                        // Keep defaults for others
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
            };

            swc_core::ecma::minifier::optimize(
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
            )
        } else {
            program
        }
    });

    // Generate minified code
    let mut buf = vec![];
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, None);
        let mut emitter = Emitter {
            cfg: swc_core::ecma::codegen::Config::default().with_minify(true),
            cm,
            comments: None,
            wr: writer,
        };

        emitter
            .emit_program(&program)
            .context("Failed to emit code")?;
    }

    let minified_code = String::from_utf8(buf).context("Failed to convert output to UTF-8")?;

    let minified_size = minified_code.len();
    let elapsed = start.elapsed();

    let stats =
        MinifyStats::with_sizes(original_size, minified_size).with_time(elapsed.as_millis());

    Ok(MinifyResult {
        code: minified_code,
        map: None, // TODO: Implement source map generation with SWC
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
