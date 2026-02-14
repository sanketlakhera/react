// Pattern: Form with onChange
// Tests controlled input pattern
function SearchForm({ onSearch }) {
    const [query, setQuery] = useState('');
    const [category, setCategory] = useState('all');

    const handleQueryChange = (e) => {
        setQuery(e.target.value);
    };

    const handleCategoryChange = (e) => {
        setCategory(e.target.value);
    };

    const handleSubmit = (e) => {
        e.preventDefault();
        onSearch({ query, category });
    };

    return (
        <form onSubmit={handleSubmit}>
            <input
                type="text"
                value={query}
                onChange={handleQueryChange}
                placeholder="Search..."
            />
            <select value={category} onChange={handleCategoryChange}>
                <option value="all">All</option>
                <option value="posts">Posts</option>
                <option value="users">Users</option>
            </select>
            <button type="submit">Search</button>
        </form>
    );
}
