/**
 * Batch Fixture Runner
 * 
 * Runs the Rust compiler against React's official fixtures.
 * Usage: node batch-runner.js <fixtures-dir> [--limit N]
 */

import fs from 'fs';
import path from 'path';
import { execSync, spawn } from 'child_process';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, '../..');

// Parse arguments
const args = process.argv.slice(2);
const fixturesDir = args[0] || path.join(__dirname, 'fixtures-react/compiler');
const limitIndex = args.indexOf('--limit');
const limit = limitIndex !== -1 ? parseInt(args[limitIndex + 1], 10) : 50;

// Find all fixture files (not expect.md or error files)
function findFixtures(dir, maxCount = 50) {
    const fixtures = [];

    const files = fs.readdirSync(dir, { withFileTypes: true });

    for (const file of files) {
        if (fixtures.length >= maxCount) break;

        const fullPath = path.join(dir, file.name);

        if (file.isDirectory()) {
            fixtures.push(...findFixtures(fullPath, maxCount - fixtures.length));
        } else if (
            (file.name.endsWith('.js') || file.name.endsWith('.ts') || file.name.endsWith('.tsx')) &&
            !file.name.includes('.expect.') &&
            !file.name.startsWith('error.')
        ) {
            fixtures.push(fullPath);
        }
    }

    return fixtures;
}

// Run the Rust compiler on a fixture
function runCompiler(fixturePath) {
    try {
        const result = execSync(
            `cargo run --quiet -- --input "${fixturePath}"`,
            {
                cwd: projectRoot,
                encoding: 'utf-8',
                timeout: 10000,
                stdio: ['pipe', 'pipe', 'pipe']
            }
        );
        return { success: true, output: result };
    } catch (error) {
        return {
            success: false,
            error: error.stderr?.toString() || error.message
        };
    }
}

// Main
console.log('='.repeat(60));
console.log('React Compiler Rust - Batch Fixture Testing');
console.log('='.repeat(60));
console.log(`Fixtures directory: ${fixturesDir}`);
console.log(`Limit: ${limit} fixtures`);
console.log('');

const fixtures = findFixtures(fixturesDir, limit);
console.log(`Found ${fixtures.length} fixtures to test\n`);

const results = {
    passed: 0,
    failed: 0,
    errors: []
};

for (const fixture of fixtures) {
    const relativePath = path.relative(fixturesDir, fixture);
    process.stdout.write(`Testing ${relativePath}... `);

    const result = runCompiler(fixture);

    if (result.success) {
        console.log('✓');
        results.passed++;
    } else {
        console.log('✗');
        results.failed++;
        results.errors.push({
            fixture: relativePath,
            error: result.error?.slice(0, 200)
        });
    }
}

console.log('\n' + '='.repeat(60));
console.log('Summary');
console.log('='.repeat(60));
console.log(`Passed: ${results.passed}/${fixtures.length}`);
console.log(`Failed: ${results.failed}/${fixtures.length}`);

if (results.errors.length > 0 && results.errors.length <= 10) {
    console.log('\nFirst 10 failures:');
    for (const err of results.errors.slice(0, 10)) {
        console.log(`  - ${err.fixture}`);
        if (err.error) {
            console.log(`    ${err.error.split('\n')[0]}`);
        }
    }
}

// Write detailed report
const reportPath = path.join(__dirname, 'batch-report.json');
fs.writeFileSync(reportPath, JSON.stringify(results, null, 2));
console.log(`\nDetailed report: ${reportPath}`);
