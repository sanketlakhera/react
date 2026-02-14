// Pattern: useMemo for expensive computation
// Tests memoization of computed values
function ExpensiveList(props) {
    const sortedItems = useMemo(() => {
        return props.items.slice().sort((a, b) => a.value - b.value);
    }, [props.items]);

    const total = useMemo(() => {
        return sortedItems.reduce((sum, item) => sum + item.value, 0);
    }, [sortedItems]);

    return (
        <div>
            <span>Total: {total}</span>
            <ul>
                {sortedItems.map(item => <li key={item.id}>{item.name}</li>)}
            </ul>
        </div>
    );
}
