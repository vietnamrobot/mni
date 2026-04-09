use mni::{Minifier, MinifyOptions};

/// Regression test for SWC collapse_vars bug
/// Issue: SWC's aggressive collapse_vars optimization produces invalid JavaScript
///
/// Example broken output:
/// t&&t.querySelector("#theme-icon")||t.textContent="dark"===e?"☀️":"🌙"
///
/// This creates an invalid left-hand side in assignment expression.
#[test]
fn test_swc_collapse_vars_bug() {
    let code = r#"
        function updateToggleIcon(theme) {
            const toggleButton = document.getElementById('theme-toggle');
            if (!toggleButton) return;

            const icon = toggleButton.querySelector('#theme-icon') || toggleButton;
            icon.textContent = theme === 'dark' ? '☀️' : '🌙';
        }
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // The minified code should still be valid JavaScript
    // Verify it contains the assignment properly separated
    assert!(result.code.contains("textContent"));

    // Verify we can parse it back (would fail if invalid JS)
    // This is the real test - if SWC produces invalid JS, this will panic
    let minifier2 = Minifier::new(MinifyOptions::default());
    let reparse = minifier2.minify_js(&result.code);
    assert!(
        reparse.is_ok(),
        "Minified output should be valid JavaScript that can be re-parsed"
    );

    // The output should NOT contain the broken pattern
    // (trying to assign to a logical OR expression)
    let broken_pattern = "||t.textContent=";
    assert!(
        !result.code.contains(broken_pattern),
        "Should not produce invalid assignment to logical OR expression"
    );
}

/// Ensure we still get good compression even with aggressive opts disabled
#[test]
fn test_compression_quality() {
    let code = r#"
        function fibonacci(n) {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        function add(a, b) {
            return a + b;
        }

        const result = add(1, 2);
        const x = 5 + 10;
        const y = 20 * 2;
        const numbers = [1, 2, 3, 4, 5];
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Should still achieve good compression
    assert!(
        result.stats.compression_ratio > 0.30,
        "Should achieve at least 30% compression, got {:.1}%",
        result.stats.compression_ratio * 100.0
    );

    // Should be valid JavaScript
    let reparse = minifier.minify_js(&result.code);
    assert!(
        reparse.is_ok(),
        "Compressed output should be valid JavaScript"
    );
}

/// Test that mangle still works with safe compression
#[test]
fn test_mangle_with_safe_compression() {
    let code = r#"
        function calculateTotal(items) {
            let total = 0;
            for (let item of items) {
                total += item.price;
            }
            return total;
        }
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Identifiers should be mangled (shorter var names)
    assert!(result.stats.minified_size < result.stats.original_size);

    // Should still be valid
    let reparse = minifier.minify_js(&result.code);
    assert!(reparse.is_ok());
}

/// Regression test for issue #1: `conditionals:true` creates invalid `&&` assignments
/// Input: `if (cond) x = y;` should NOT produce `cond&&x=y;` (invalid JS)
/// It should produce `cond&&(x=y)` or keep the if-statement form.
#[test]
fn test_conditional_assignment_produces_valid_js() {
    let code = r#"let a = true; let b = 2.5; var c; if (a) c = b;"#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // The output must be re-parseable as valid JavaScript
    let reparse = minifier.minify_js(&result.code);
    assert!(
        reparse.is_ok(),
        "Minified output should be valid JS, got: {}",
        result.code
    );

    // Specifically: `&&c=b` without parens is invalid — the assignment must be wrapped
    assert!(
        !result.code.contains("&&c=b"),
        "Should not produce bare assignment after &&, got: {}",
        result.code
    );
}

/// Regression test for issue #1 variant: assignment in || expression
#[test]
fn test_conditional_or_assignment_produces_valid_js() {
    let code = r#"let a = false; let b = 2.5; var c; if (!a) c = b;"#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // The output must be re-parseable as valid JavaScript
    let reparse = minifier.minify_js(&result.code);
    assert!(
        reparse.is_ok(),
        "Minified output should be valid JS, got: {}",
        result.code
    );
}

/// Test conditional optimization doesn't break
#[test]
fn test_conditional_optimization_safe() {
    let code = r#"
        function check(value) {
            if (value === true) {
                return "yes";
            }
            if (value === false) {
                return "no";
            }
            return "maybe";
        }
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Should optimize conditionals
    assert!(result.stats.compression_ratio > 0.0);

    // Should be valid
    let reparse = minifier.minify_js(&result.code);
    assert!(reparse.is_ok());
}
