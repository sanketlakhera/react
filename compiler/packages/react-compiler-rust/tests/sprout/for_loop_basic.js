function sum(n) {
    let total = 0;
    for (let i = 0; i < n; i++) {
        total = total + i;
    }
    return total;
}

const FIXTURE_ENTRYPOINT = {
    fn: sum,
    params: [10],
};
