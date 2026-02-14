// Pattern: useCallback for stable function references
// Tests callback memoization
function TodoList(props) {
    const [todos, setTodos] = useState([]);

    const addTodo = useCallback((text) => {
        setTodos(prev => [...prev, { id: Date.now(), text }]);
    }, []);

    const removeTodo = useCallback((id) => {
        setTodos(prev => prev.filter(t => t.id !== id));
    }, []);

    return (
        <div>
            <TodoInput onAdd={addTodo} />
            {todos.map(todo => (
                <TodoItem
                    key={todo.id}
                    todo={todo}
                    onRemove={removeTodo}
                />
            ))}
        </div>
    );
}
