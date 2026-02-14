// Pattern: Children prop
// Tests children rendering pattern
function Card({ title, children, footer }) {
    return (
        <div className="card">
            <div className="card-header">
                <h2>{title}</h2>
            </div>
            <div className="card-body">
                {children}
            </div>
            {footer && (
                <div className="card-footer">
                    {footer}
                </div>
            )}
        </div>
    );
}
