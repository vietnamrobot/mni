use mni::{Minifier, MinifyOptions};

#[test]
fn test_js_minification() {
    let code = r#"
        function add(a, b) {
            return a + b;
        }
        const result = add(1, 2);
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Should be minified
    assert!(result.stats.minified_size < result.stats.original_size);
    assert!(result.stats.compression_ratio > 0.0);

    // Should contain function definition (mangled or not)
    assert!(result.code.contains("function") || result.code.contains("add"));
}

#[test]
fn test_css_minification() {
    let css = r#"
        body {
            margin: 0;
            padding: 0;
            color: #ffffff;
        }
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_css(css).unwrap();

    // Should be minified
    assert!(result.stats.minified_size < result.stats.original_size);

    // Should contain optimized color
    assert!(result.code.contains("#fff") || result.code.contains("white"));
}

#[test]
fn test_json_minification() {
    let json = r#"
        {
            "name": "test",
            "version": "1.0.0",
            "nested": {
                "key": "value"
            }
        }
    "#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_json(json).unwrap();

    // Should be minified
    assert!(result.stats.minified_size < result.stats.original_size);

    // Should still be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&result.code).unwrap();
    assert_eq!(parsed["name"], "test");
    assert_eq!(parsed["version"], "1.0.0");
}

#[test]
fn test_development_preset() {
    let code = "function test() { console.log('test'); }";

    let options = MinifyOptions::development();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Development mode should not mangle
    assert!(result.code.contains("test"));
}

#[test]
fn test_production_preset() {
    let code = "function testFunction() { return 42; }";

    let options = MinifyOptions::production();
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Production should minify aggressively
    assert!(result.stats.compression_ratio > 0.0);
}

#[test]
fn test_auto_detect_json() {
    let json = r#"{"test": true}"#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_auto(json, Some("test.json")).unwrap();

    assert!(result.code.contains("test"));
    assert!(result.code.contains("true"));
}

#[test]
fn test_auto_detect_css() {
    let css = "body { color: red; }";

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_auto(css, Some("test.css")).unwrap();

    assert!(result.code.contains("body"));
    assert!(result.code.contains("red"));
}

#[test]
fn test_constant_folding() {
    let code = r#"
        const x = 5 + 10;
        const y = 20 * 2;
    "#;

    let options = MinifyOptions {
        compress: true,
        ..Default::default()
    };
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Constants should be folded
    assert!(result.code.contains("15") || result.code.contains("40"));
}

#[test]
fn test_dead_code_elimination() {
    let code = r#"
        if (false) {
            console.log("dead code");
        }
        if (true) {
            console.log("alive");
        }
    "#;

    let options = MinifyOptions {
        compress: true,
        ..Default::default()
    };
    let minifier = Minifier::new(options);
    let result = minifier.minify_js(code).unwrap();

    // Dead code should be removed
    assert!(!result.code.contains("dead code"));
}
