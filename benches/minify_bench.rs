use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mni::{Minifier, MinifyOptions};

fn bench_minify_js(c: &mut Criterion) {
    let code = r#"
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }

        class Calculator {
            constructor() {
                this.result = 0;
            }

            add(a, b) {
                this.result = a + b;
                return this.result;
            }

            multiply(a, b) {
                this.result = a * b;
                return this.result;
            }
        }

        const numbers = [1, 2, 3, 4, 5];
        const doubled = numbers.map(n => n * 2);
        console.log(doubled);
    "#;

    c.bench_function("minify_js_basic", |b| {
        b.iter(|| {
            let options = MinifyOptions::default();
            let minifier = Minifier::new(options);
            minifier.minify_js(black_box(code)).unwrap()
        });
    });
}

fn bench_minify_json(c: &mut Criterion) {
    let json = r#"{
        "name": "test",
        "version": "1.0.0",
        "dependencies": {
            "react": "^18.0.0",
            "react-dom": "^18.0.0"
        },
        "scripts": {
            "build": "webpack",
            "test": "jest"
        }
    }"#;

    c.bench_function("minify_json_basic", |b| {
        b.iter(|| {
            let options = MinifyOptions::default();
            let minifier = Minifier::new(options);
            minifier.minify_json(black_box(json)).unwrap()
        });
    });
}

criterion_group!(benches, bench_minify_js, bench_minify_json);
criterion_main!(benches);
