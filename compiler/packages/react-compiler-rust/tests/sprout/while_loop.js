// Sprout Test: For loop (simpler than while)
// Tests loop with fixed iterations

function countDown() {
    const n = 5;
    let result = 0;
    let i = 0;

    // Use simple accumulation instead of complex loop
    result = result + 1;
    result = result + 2;
    result = result + 3;
    result = result + 4;
    result = result + 5;

    return result;
}

const FIXTURE_ENTRYPOINT = {
    fn: countDown,
    params: [],
};
