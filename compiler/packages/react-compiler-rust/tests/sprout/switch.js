
function test_basic(x) {
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

function test_fallthrough(x) {
    let res = 0;
    switch (x) {
        case 1:
            res += 1;
        // fallthrough
        case 2:
            res += 2;
            break;
        case 3:
            res += 4;
            break;
    }
    return res;
}

function test_nested() {
    let res = 0;
    for (let i = 0; i < 3; i++) {
        switch (i) {
            case 0:
                res += 1;
                break;
            case 1:
                res += 10;
                continue; // Should continue the loop
            case 2:
                res += 100;
                break;
        }
    }
    return res;
}

// Global entry point
function main() {
    return {
        basic_1: test_basic(1),
        basic_2: test_basic(2),
        basic_def: test_basic(99),
        ft_1: test_fallthrough(1), // Expect 3
        ft_2: test_fallthrough(2), // Expect 2
        ft_3: test_fallthrough(3), // Expect 4
        nested: test_nested(),     // Expect 111
    };
}

const FIXTURE_ENTRYPOINT = {
    fn: main,
    params: [],
};
