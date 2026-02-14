// Sprout Test: Simple conditional (single if/else)
// Tests basic if/else branch with return

function checkValue() {
    const value = 85;
    const threshold = 80;
    const isAbove = value > threshold;
    return isAbove;
}

const FIXTURE_ENTRYPOINT = {
    fn: checkValue,
    params: [],
};
