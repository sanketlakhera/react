// Pattern: Props destructuring
// Tests prop destructuring with defaults
function UserCard({ name, email, role = 'user', avatar }) {
    const initials = name.split(' ').map(n => n[0]).join('');

    return (
        <div className="user-card">
            {avatar ? (
                <img src={avatar} alt={name} />
            ) : (
                <div className="initials">{initials}</div>
            )}
            <h3>{name}</h3>
            <p>{email}</p>
            <span className="role">{role}</span>
        </div>
    );
}
