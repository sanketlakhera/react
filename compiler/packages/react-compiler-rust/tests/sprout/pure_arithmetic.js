// Sprout Test: Pure literal computation
// Tests computation with inline values (no parameters needed)

function compute() {
    const a = 5;
    const b = 10;
    const c = 3;
    const sum = a + b;
    const product = sum * c;
    const result = product - a;
    return result;
}

const FIXTURE_ENTRYPOINT = {
    fn: compute,
    params: [],
};
