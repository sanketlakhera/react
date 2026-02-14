/**
 * E2E Test: Counter Component
 * 
 * Tests a basic counter with click interactions.
 */

import React, { useState } from 'react';

// The component to test
export function Component() {
    const [count, setCount] = useState(0);

    return React.createElement('div', null,
        React.createElement('span', { id: 'count' }, count),
        React.createElement('button', {
            id: 'increment',
            onClick: () => setCount(count + 1)
        }, 'Increment')
    );
}

// Test cases
export const tests = [
    {
        name: 'initial render shows 0',
        run: (document) => {
            return document.getElementById('count').textContent;
        },
        expect: '0',
    },
    {
        name: 'clicking increment updates count',
        run: async (document, window) => {
            const button = document.getElementById('increment');
            button.click();
            // Wait for React to update
            await new Promise(r => setTimeout(r, 10));
            return document.getElementById('count').textContent;
        },
        expect: '1',
    },
];
