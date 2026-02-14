function updateEx(x) {
    let a = x;
    let b = ++a; // pre-increment: a=11, b=11
    let c = a++; // post-increment: a=12, c=11
    let d = --a; // pre-decrement: a=11, d=11
    let e = a--; // post-decrement: a=10, e=11

    return { a, b, c, d, e };
}

const FIXTURE_ENTRYPOINT = {
    fn: updateEx,
    params: [10],
};
