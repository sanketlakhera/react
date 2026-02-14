// Test switch performance with release build
use std::time::Instant;
use react_compiler_rust::compile;
use oxc_span::SourceType;

fn main() {
    // Test various switch scenarios
    let test_cases = vec![
        ("Basic Switch (3 cases)", r#"
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
"#),
        ("Switch with 20 cases", r#"
function manyCasesSwitch(x) {
    let result = 0;
    switch (x) {
        case 1: result = 10; break;
        case 2: result = 20; break;
        case 3: result = 30; break;
        case 4: result = 40; break;
        case 5: result = 50; break;
        case 6: result = 60; break;
        case 7: result = 70; break;
        case 8: result = 80; break;
        case 9: result = 90; break;
        case 10: result = 100; break;
        case 11: result = 110; break;
        case 12: result = 120; break;
        case 13: result = 130; break;
        case 14: result = 140; break;
        case 15: result = 150; break;
        case 16: result = 160; break;
        case 17: result = 170; break;
        case 18: result = 180; break;
        case 19: result = 190; break;
        case 20: result = 200; break;
        default: result = 0;
    }
    return result;
}
"#),
        ("Switch with fallthrough", r#"
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
"#),
        ("Complex switch with nested control flow", r#"
function complexSwitch(x) {
    let result = 0;
    switch (x) {
        case 1:
            for (let i = 0; i < 5; i++) {
                result += i;
            }
            break;
        case 2:
            if (result > 0) {
                result *= 2;
            } else {
                result = 10;
            }
            break;
        case 3:
            while (result < 100) {
                result += 10;
            }
            break;
        default:
            result = -1;
    }
    return result;
}
"#)
    ];

    println!("React Compiler Rust - Performance Analysis (Release Build)");
    println!("========================================================");

    for (name, code) in test_cases {
        // Warmup run
        let _ = compile(code, SourceType::mjs()).unwrap();
        
        // Timing runs
        const ITERATIONS: usize = 50;
        let start = Instant::now();
        
        for _ in 0..ITERATIONS {
            let _ = compile(code, SourceType::mjs()).unwrap();
        }
        
        let total_time = start.elapsed();
        let avg_time = total_time.as_micros() as f64 / ITERATIONS as f64;
        
        println!("\n{}:", name);
        println!("  Average compilation time: {:.2} μs", avg_time);
        println!("  Total time for {} iterations: {:?}", ITERATIONS, total_time);
        println!("  Throughput: {:.2} compiles/sec", ITERATIONS as f64 / total_time.as_secs_f64());
    }
}