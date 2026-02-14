#!/usr/bin/env python3

"""
Profiling script for React Compiler Rust
This script runs the compiler multiple times and analyzes performance
"""

import subprocess
import time
import json
import tempfile
import os

def run_compiler_with_timing(code, iterations=100):
    """Run the compiler multiple times and collect timing data"""
    times = []
    
    # Create temporary JavaScript file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
        f.write(code)
        temp_file = f.name
    
    try:
        for _ in range(iterations):
            start_time = time.time()
            result = subprocess.run([
                'cargo', 'run', '--release', '--bin', 'react-compiler-rust', '--', '--input', temp_file
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0:
                elapsed = time.time() - start_time
                times.append(elapsed)
            else:
                print(f"Compilation failed: {result.stderr}")
                return None
    finally:
        os.unlink(temp_file)
    
    return times

def main():
    test_cases = [
        ("Basic Switch (3 cases)", """
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
"""),
        ("Switch with 50 cases", """
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
        case 21: result = 210; break;
        case 22: result = 220; break;
        case 23: result = 230; break;
        case 24: result = 240; break;
        case 25: result = 250; break;
        case 26: result = 260; break;
        case 27: result = 270; break;
        case 28: result = 280; break;
        case 29: result = 290; break;
        case 30: result = 300; break;
        case 31: result = 310; break;
        case 32: result = 320; break;
        case 33: result = 330; break;
        case 34: result = 340; break;
        case 35: result = 350; break;
        case 36: result = 360; break;
        case 37: result = 370; break;
        case 38: result = 380; break;
        case 39: result = 390; break;
        case 40: result = 400; break;
        case 41: result = 410; break;
        case 42: result = 420; break;
        case 43: result = 430; break;
        case 44: result = 440; break;
        case 45: result = 450; break;
        case 46: result = 460; break;
        case 47: result = 470; break;
        case 48: result = 480; break;
        case 49: result = 490; break;
        case 50: result = 500; break;
        default: result = 0;
    }
    return result;
}
"""),
        ("Switch with fallthrough", """
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
""")
    ]
    
    print("React Compiler Rust - Profiling Results")
    print("======================================")
    
    for name, code in test_cases:
        print(f"\nBenchmark: {name}")
        print("-" * 50)
        
        times = run_compiler_with_timing(code, iterations=50)  # Reduced for faster testing
        if times:
            avg_time = sum(times) / len(times)
            min_time = min(times)
            max_time = max(times)
            
            print(f"  Average time: {avg_time*1000:.2f} ms")
            print(f"  Min time: {min_time*1000:.2f} ms")
            print(f"  Max time: {max_time*1000:.2f} ms")
            print(f"  Std dev: {((sum((t - avg_time)**2 for t in times) / len(times))**0.5)*1000:.2f} ms")
            print(f"  Throughput: {1/avg_time:.2f} compiles/sec")

if __name__ == "__main__":
    main()