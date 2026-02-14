// A simple sprout test fixture with FIXTURE_ENTRYPOINT
function add(a, b) {
    return a + b;
}

const FIXTURE_ENTRYPOINT = {
    fn: add,
    params: [10, 20],
};
