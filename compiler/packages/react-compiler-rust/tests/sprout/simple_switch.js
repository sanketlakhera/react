// Sprout Test: Simple Switch Statement

function simpleSwitch(x) {
    let res = 0;
    switch (x) {
        case 1:
            res = 10;
            break;
        case 2:
            res = 20;
            break;
        default:
            res = 30;
    }
    return res;
}

const FIXTURE_ENTRYPOINT = {
    fn: simpleSwitch,
    params: [1],
};