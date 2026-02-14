// Pattern: Early returns
// Tests early return optimization
function UserProfile({ user, isLoading, error }) {
    if (isLoading) {
        return <div className="loading">Loading...</div>;
    }

    if (error) {
        return <div className="error">{error.message}</div>;
    }

    if (!user) {
        return <div className="empty">No user found</div>;
    }

    return (
        <div className="profile">
            <h1>{user.name}</h1>
            <p>{user.bio}</p>
        </div>
    );
}
