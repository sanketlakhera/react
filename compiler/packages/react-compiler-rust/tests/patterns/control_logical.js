// Pattern: Logical operators for conditional rendering
// Tests && and || operators
function Notification({ message, type, onDismiss }) {
    const isError = type === 'error';
    const isWarning = type === 'warning';

    return (
        <div className={`notification notification-${type}`}>
            {isError && <span className="icon">⚠️</span>}
            {isWarning && <span className="icon">⚡</span>}

            <p>{message || 'No message'}</p>

            {onDismiss && (
                <button onClick={onDismiss}>
                    Dismiss
                </button>
            )}
        </div>
    );
}
