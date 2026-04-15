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
fn test_html_minification_strips_whitespace_and_comments() {
    let html = r#"<!DOCTYPE html>
<html>
    <head>
        <title>Hello</title>
        <!-- this comment should go -->
    </head>
    <body>
        <p>   hello   world   </p>
    </body>
</html>
"#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_html(html).unwrap();

    assert!(result.stats.minified_size < result.stats.original_size);
    assert!(!result.code.contains("this comment should go"));
    assert!(result.code.contains("Hello"));
    assert!(result.code.contains("hello"));
    assert!(result.code.contains("world"));
}

#[test]
fn test_html_minification_inlines_js_and_css() {
    let html = r#"<!DOCTYPE html>
<html>
    <head>
        <style>
            body {
                color: #ffffff;
                margin: 0px;
            }
        </style>
    </head>
    <body>
        <script>
            const   x   =   1   +   2;
            console.log(x);
        </script>
    </body>
</html>
"#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_html(html).unwrap();

    // Inline CSS should be minified (color collapsed to #fff and 0px → 0).
    assert!(result.code.contains("#fff"));
    assert!(!result.code.contains("0px"));
    // Inline JS should be minified (whitespace between tokens removed).
    assert!(!result.code.contains("const   x"));
}

#[test]
fn test_svg_minification_strips_whitespace_and_comments() {
    let svg = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
    <!-- this comment should go -->
    <title>  My Icon  </title>
    <rect x="10" y="10" width="80" height="80" fill="#ffffff"/>
    <circle cx="50" cy="50" r="30" fill="red"/>
</svg>
"##;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_svg(svg).unwrap();

    assert!(result.stats.minified_size < result.stats.original_size);
    assert!(!result.code.contains("this comment should go"));
    // SVG root element preserved.
    assert!(result.code.contains("<svg"));
    // Drawn elements survive — oxvg may rewrite <rect> as <path> (a semantic-
    // preserving shape conversion), so accept either, but the circle should
    // stay as a circle under the safe preset.
    assert!(result.code.contains("<rect") || result.code.contains("<path"));
    assert!(result.code.contains("<circle"));
    // Color minification should apply: #ffffff → #fff.
    assert!(!result.code.contains("#ffffff"));
    assert!(result.code.contains("#fff"));
}

#[test]
fn test_svg_minification_preserves_valid_xml_roundtrip() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
    <path d="M12 2 L22 22 L2 22 Z" fill="blue"/>
</svg>"#;

    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_svg(svg).unwrap();

    // Re-parse to prove the output is well-formed XML.
    let doc = roxmltree::Document::parse(&result.code)
        .expect("minified SVG output must re-parse as valid XML");
    let root = doc.root_element();
    assert_eq!(root.tag_name().name(), "svg");

    // The <path> element must survive with non-empty `d` data.
    let path = root
        .descendants()
        .find(|n| n.has_tag_name("path"))
        .expect("path element should be preserved");
    let d = path
        .attribute("d")
        .expect("path must keep its `d` attribute");
    assert!(!d.is_empty(), "path data was emptied");
    // Path data must still start with a moveto command (M or m).
    assert!(
        d.trim_start().starts_with('M') || d.trim_start().starts_with('m'),
        "unexpected path data: {d}"
    );
}

#[test]
fn test_auto_detect_svg_by_extension() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1"/></svg>"#;
    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_auto(svg, Some("icon.svg")).unwrap();

    assert!(result.code.contains("<svg"));
    // Shape survives in some form (rect or converted path).
    assert!(result.code.contains("<rect") || result.code.contains("<path"));
}

#[test]
fn test_auto_detect_html() {
    let html = "<!DOCTYPE html><html><body><h1>  Title  </h1></body></html>";
    let options = MinifyOptions::default();
    let minifier = Minifier::new(options);
    let result = minifier.minify_auto(html, Some("page.html")).unwrap();

    assert!(result.code.contains("<h1>"));
    assert!(result.stats.minified_size < result.stats.original_size);
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
