
function test_break() {
    let sum = 0;
    for (let i = 0; i < 10; i++) {
        if (i === 5) {
            break;
        }
        sum += i;
    }
    return sum; // Should be 0+1+2+3+4 = 10
}

function test_continue() {
    let sum = 0;
    for (let i = 0; i < 5; i++) {
        if (i === 2) {
            continue;
        }
        sum += i;
    }
    return sum; // Should be 0+1+3+4 = 8
}

function test_nested() {
    let count = 0;
    for (let i = 0; i < 3; i++) {
        for (let j = 0; j < 3; j++) {
            if (i === 1) break; // Exits inner loop
            if (j === 1) continue; // Skips inner loop iteration
            count++;
        }
    }
    // i=0: j=0 (count=1), j=1 (skip), j=2 (count=2) -> count=2
    // i=1: j=0 (break) -> count=2
    // i=2: j=0 (count=3), j=1 (skip), j=2 (count=4) -> count=4
    return count;
}


function test_all() {
    return {
        break: test_break(),
        continue: test_continue(),
        nested: test_nested(),
    };
}

const FIXTURE_ENTRYPOINT = {
    fn: test_all,
    params: [],
};
