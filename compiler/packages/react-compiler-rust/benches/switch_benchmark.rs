//! Benchmark suite for the React Compiler Rust implementation, with focus on switch statements

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use react_compiler_rust::compile;
use oxc_span::SourceType;

fn benchmark_switch_simple(c: &mut Criterion) {
    let code = r#"
function simpleSwitch(x) {
    let res = 0;
    switch (x) {
        case 1:
            res = 10;
            break;
        case 2:
            res = 20;
            break;
        case 3:
            res = 30;
            break;
        case 4:
            res = 40;
            break;
        case 5:
            res = 50;
            break;
        default:
            res = 0;
    }
    return res;
}
"#;

    c.bench_function("switch_simple", |b| {
        b.iter(|| {
            let result = compile(black_box(code), SourceType::mjs());
            black_box(result).unwrap();
        })
    });
}

fn benchmark_switch_many_cases(c: &mut Criterion) {
    let code = r#"
function switchManyCases(x) {
    let res = 0;
    switch (x) {
        case 1: res = 1; break;
        case 2: res = 2; break;
        case 3: res = 3; break;
        case 4: res = 4; break;
        case 5: res = 5; break;
        case 6: res = 6; break;
        case 7: res = 7; break;
        case 8: res = 8; break;
        case 9: res = 9; break;
        case 10: res = 10; break;
        case 11: res = 11; break;
        case 12: res = 12; break;
        case 13: res = 13; break;
        case 14: res = 14; break;
        case 15: res = 15; break;
        case 16: res = 16; break;
        case 17: res = 17; break;
        case 18: res = 18; break;
        case 19: res = 19; break;
        case 20: res = 20; break;
        default: res = 0;
    }
    return res;
}
"#;

    c.bench_function("switch_many_cases", |b| {
        b.iter(|| {
            let result = compile(black_box(code), SourceType::mjs());
            black_box(result).unwrap();
        })
    });
}

fn benchmark_switch_fallthrough(c: &mut Criterion) {
    let code = r#"
function switchFallthrough(x) {
    let res = 0;
    switch (x) {
        case 1:
            res += 1;
        case 2:
            res += 2;
            break;
        case 3:
            res += 4;
        case 4:
            res += 8;
        case 5:
            res += 16;
            break;
        default:
            res = -1;
    }
    return res;
}
"#;

    c.bench_function("switch_fallthrough", |b| {
        b.iter(|| {
            let result = compile(black_box(code), SourceType::mjs());
            black_box(result).unwrap();
        })
    });
}

// Compare switch vs if-else performance
fn benchmark_if_else_equivalent(c: &mut Criterion) {
    let code = r#"
function ifElseEquivalent(x) {
    let res = 0;
    if (x === 1) {
        res = 10;
    } else if (x === 2) {
        res = 20;
    } else if (x === 3) {
        res = 30;
    } else if (x === 4) {
        res = 40;
    } else if (x === 5) {
        res = 50;
    } else {
        res = 0;
    }
    return res;
}
"#;

    c.bench_function("if_else_equivalent", |b| {
        b.iter(|| {
            let result = compile(black_box(code), SourceType::mjs());
            black_box(result).unwrap();
        })
    });
}

// Benchmark compilation of the existing switch test
fn benchmark_existing_switch_test(c: &mut Criterion) {
    let code = r#"
function test_basic(x) {
    let res = 0;
    switch (x) {
        case 1:
            res = 10;
            break;
        case 2:
            res = 20;
            break;
        default:
            res = 30;
    }
    return res;
}

function test_fallthrough(x) {
    let res = 0;
    switch (x) {
        case 1:
            res += 1;
        // fallthrough
        case 2:
            res += 2;
            break;
        case 3:
            res += 4;
            break;
    }
    return res;
}

function test_nested() {
    let res = 0;
    for (let i = 0; i < 3; i++) {
        switch (i) {
            case 0:
                res += 1;
                break;
            case 1:
                res += 10;
                continue; // Should continue the loop
            case 2:
                res += 100;
                break;
        }
    }
    return res;
}
"#;

    c.bench_function("existing_switch_test", |b| {
        b.iter(|| {
            let result = compile(black_box(code), SourceType::mjs());
            black_box(result).unwrap();
        })
    });
}

criterion_group!(
    name = switch_benchmarks;
    config = Criterion::default().sample_size(100);
    targets = 
        benchmark_switch_simple,
        benchmark_switch_many_cases,
        benchmark_switch_fallthrough,
        benchmark_if_else_equivalent,
        benchmark_existing_switch_test
);
criterion_main!(switch_benchmarks);