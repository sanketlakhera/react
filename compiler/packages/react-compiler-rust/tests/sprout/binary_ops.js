// Sprout Test: Binary operations on literals
// Tests all binary operators work correctly

function binaryOps() {
    const a = 10;
    const b = 3;

    const add = a + b;
    const sub = a - b;
    const mul = a * b;
    const lt = a < b;
    const gt = a > b;

    return add + sub + mul;
}

const FIXTURE_ENTRYPOINT = {
    fn: binaryOps,
    params: [],
};
