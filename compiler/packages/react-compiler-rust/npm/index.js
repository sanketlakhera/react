// @ts-check
const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const { platform, arch } = process;

let nativeBinding = null;
let localFileExisted = false;
let loadError = null;

// Try to load the native binding
function loadBinding() {
    // Try platform-specific binary
    const bindingPath = join(__dirname, `react-compiler-rust.${platform}-${arch}.node`);

    if (existsSync(bindingPath)) {
        try {
            return require(bindingPath);
        } catch (e) {
            loadError = e;
        }
    }

    // Try generic binding (for local development)
    const genericPath = join(__dirname, 'react-compiler-rust.node');
    if (existsSync(genericPath)) {
        try {
            return require(genericPath);
        } catch (e) {
            loadError = e;
        }
    }

    // Try loading from parent directory (when building)
    const parentPath = join(__dirname, '..', 'react-compiler-rust.node');
    if (existsSync(parentPath)) {
        try {
            return require(parentPath);
        } catch (e) {
            loadError = e;
        }
    }

    return null;
}

nativeBinding = loadBinding();

if (!nativeBinding) {
    if (loadError) {
        throw loadError;
    }
    throw new Error(
        `Failed to load native binding. Platform: ${platform}, Arch: ${arch}`
    );
}

module.exports = nativeBinding;
