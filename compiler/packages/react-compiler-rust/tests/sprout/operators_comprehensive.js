function testOperators(a, b) {
    const bitwiseAnd = a & b;
    const bitwiseOr = a | b;
    const bitwiseXor = a ^ b;
    const leftShift = a << 1;
    const rightShift = a >> 1;
    const plus = +a;
    const bitwiseNot = ~a;

    return {
        bitwiseAnd,
        bitwiseOr,
        bitwiseXor,
        leftShift,
        rightShift,
        plus,
        bitwiseNot
    };
}

const FIXTURE_ENTRYPOINT = {
    fn: testOperators,
    params: [5, 3],
};
