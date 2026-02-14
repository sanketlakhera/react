// Pattern: Array.filter for filtered lists
// Tests filter + map combination
function TaskBoard({ tasks, filter }) {
    const filteredTasks = tasks.filter(task => {
        if (filter === 'completed') return task.completed;
        if (filter === 'active') return !task.completed;
        return true;
    });

    const taskCount = filteredTasks.length;

    return (
        <div className="task-board">
            <h2>Tasks ({taskCount})</h2>
            <ul>
                {filteredTasks.map(task => (
                    <li key={task.id} className={task.completed ? 'done' : ''}>
                        {task.title}
                    </li>
                ))}
            </ul>
        </div>
    );
}
