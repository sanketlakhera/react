use std::time::Instant;
use react_compiler_rust::compile;
use oxc_span::SourceType;

fn main() {
    println!("React Compiler Rust - Performance Analysis");
    println!("========================================");

    // Test 1: Basic switch
    let basic_switch_code = r#"
function basicSwitch(x) {
    let result = 0;
    switch (x) {
        case 1: result = 10; break;
        case 2: result = 20; break;
        case 3: result = 30; break;
        default: result = 0;
    }
    return result;
}
"#;

    // Test 2: Switch with many cases
    let mut many_cases_code = String::from("function manyCasesSwitch(x) {\n    let result = 0;\n    switch (x) {\n");
    for i in 1..=50 {
        many_cases_code.push_str(&format!("        case {}: result = {}; break;\n", i, i * 10));
    }
    many_cases_code.push_str("        default: result = -1;\n    }\n    return result;\n}\n");

    // Test 3: Switch with fallthrough
    let fallthrough_switch_code = r#"
function fallthroughSwitch(x) {
    let result = 0;
    switch (x) {
        case 1:
            result += 10;
        case 2:
            result += 20;
            break;
        case 3:
            result += 30;
        case 4:
            result += 40;
            break;
        default:
            result = 0;
    }
    return result;
}
"#;

    // Run benchmarks
    run_benchmark("Basic Switch (3 cases)", basic_switch_code);
    run_benchmark("Switch with 50 cases", &many_cases_code);
    run_benchmark("Switch with fallthrough", fallthrough_switch_code);

    println!("\nPerformance testing completed!");
}

fn run_benchmark(name: &str, code: &str) {
    const ITERATIONS: usize = 1000;
    
    println!("\nBenchmark: {}", name);
    println!("Iterations: {}", ITERATIONS);
    
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = compile(code, SourceType::mjs()).unwrap();
    }
    let duration = start.elapsed();
    
    let avg_time = duration.as_nanos() as f64 / ITERATIONS as f64;
    println!("  Total time: {:?}", duration);
    println!("  Average time: {:.2} ns per compilation", avg_time);
    println!("  Throughput: {:.2} compiles/sec", ITERATIONS as f64 / duration.as_secs_f64());
}