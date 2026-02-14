// Pattern: useEffect with dependencies
// Tests effect cleanup and dependency tracking
function DataFetcher(props) {
    const [data, setData] = useState(null);

    useEffect(() => {
        const controller = new AbortController();

        fetch(props.url, { signal: controller.signal })
            .then(res => res.json())
            .then(setData);

        return () => controller.abort();
    }, [props.url]);

    return <div>{data ? data.title : 'Loading...'}</div>;
}
