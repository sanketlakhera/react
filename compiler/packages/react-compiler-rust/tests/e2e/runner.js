/**
 * E2E Test Runner
 * 
 * Runs React components in jsdom and verifies behavior.
 * Usage: node runner.js <fixture-path>
 */

import { JSDOM } from 'jsdom';
import React from 'react';
import { createRoot } from 'react-dom/client';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Setup jsdom environment
const dom = new JSDOM('<!DOCTYPE html><html><body><div id="root"></div></body></html>', {
    url: 'http://localhost',
    pretendToBeVisual: true,
});

// Make DOM globals available (Node.js 24+ compatible)
Object.defineProperty(globalThis, 'window', { value: dom.window, writable: true, configurable: true });
Object.defineProperty(globalThis, 'document', { value: dom.window.document, writable: true, configurable: true });
Object.defineProperty(globalThis, 'navigator', { value: dom.window.navigator, writable: true, configurable: true });

// Mock React cache for useMemoCache pattern
globalThis._c = (size) => {
    const cache = new Array(size).fill(Symbol.for('react.memo_cache_sentinel'));
    return cache;
};

/**
 * Run a single E2E test
 */
async function runTest(fixturePath) {
    // Import the fixture as a proper ES module using file URL
    const absolutePath = path.resolve(fixturePath);
    const fileUrl = pathToFileURL(absolutePath).href;

    const testModule = await import(fileUrl);

    const { Component, tests } = testModule;

    if (!Component) {
        console.log(JSON.stringify({ success: false, error: 'No Component exported' }));
        return;
    }

    if (!tests || !Array.isArray(tests)) {
        console.log(JSON.stringify({ success: false, error: 'No tests array exported' }));
        return;
    }

    const results = [];

    for (const test of tests) {
        const { name, run, expect: expected } = test;

        try {
            // Clear the root
            document.getElementById('root').innerHTML = '';

            // Render the component
            const root = createRoot(document.getElementById('root'));
            root.render(React.createElement(Component));

            // Wait for render
            await new Promise(resolve => setTimeout(resolve, 0));

            // Run the test
            const actual = await run(document, dom.window);

            // Compare
            const passed = JSON.stringify(actual) === JSON.stringify(expected);

            results.push({
                name,
                passed,
                expected,
                actual,
            });

            // Cleanup
            root.unmount();
        } catch (error) {
            results.push({
                name,
                passed: false,
                error: error.message,
            });
        }
    }

    const allPassed = results.every(r => r.passed);
    console.log(JSON.stringify({ success: allPassed, results }));
}

// Main
const fixturePath = process.argv[2];
if (!fixturePath) {
    console.log(JSON.stringify({ success: false, error: 'No fixture path provided' }));
    process.exit(1);
}

runTest(fixturePath).catch(error => {
    console.log(JSON.stringify({ success: false, error: error.message }));
    process.exit(1);
});
