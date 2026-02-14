// Pattern: Ternary expressions
// Tests conditional rendering with ternary
function StatusBadge({ status, count }) {
    const color = status === 'active' ? 'green' : status === 'pending' ? 'yellow' : 'gray';
    const label = status === 'active' ? 'Active' : status === 'pending' ? 'Pending' : 'Inactive';

    return (
        <span
            className={`badge badge-${color}`}
            title={`${count} items`}
        >
            {label}
            {count > 0 ? ` (${count})` : null}
        </span>
    );
}
