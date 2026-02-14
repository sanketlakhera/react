// Pattern: onClick event handlers
// Tests event handler patterns
function ToggleButton({ label, initialState }) {
    const [isOn, setIsOn] = useState(initialState);

    const handleClick = () => {
        setIsOn(!isOn);
    };

    const handleDoubleClick = () => {
        setIsOn(initialState);
    };

    return (
        <button
            onClick={handleClick}
            onDoubleClick={handleDoubleClick}
            className={isOn ? 'active' : 'inactive'}
        >
            {label}: {isOn ? 'ON' : 'OFF'}
        </button>
    );
}
