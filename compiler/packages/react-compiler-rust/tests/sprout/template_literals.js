
function test_simple() {
    const name = "World";
    return `Hello, ${name}!`;
}

function test_multi_expr() {
    const a = 10;
    const b = 20;
    return `${a} + ${b} = ${a + b}`;
}

function test_no_interpolation() {
    return `just a plain string`;
}

function test_empty() {
    return ``;
}

function test_nested() {
    const x = 5;
    return `result: ${x > 3 ? "big" : "small"}`;
}

function test_escape() {
    return `line1\nline2\ttab`;
}

// Global entry point
function main() {
    return {
        simple: test_simple(),
        multi: test_multi_expr(),
        plain: test_no_interpolation(),
        empty: test_empty(),
        nested: test_nested(),
        escape: test_escape(),
    };
}

const FIXTURE_ENTRYPOINT = {
    fn: main,
    params: [],
};
