// Pattern: useState basic
// Tests basic state management with useState hook
function Counter() {
    const [count, setCount] = useState(0);

    const increment = () => {
        setCount(count + 1);
    };

    return (
        <div>
            <span>{count}</span>
            <button onClick={increment}>+</button>
        </div>
    );
}
