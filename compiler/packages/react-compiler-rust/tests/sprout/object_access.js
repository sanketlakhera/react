// Sprout Test: String concatenation
// Tests string operations

function greeting() {
    const first = "Hello";
    const second = "World";
    const result = first + " " + second;
    return result;
}

const FIXTURE_ENTRYPOINT = {
    fn: greeting,
    params: [],
};
