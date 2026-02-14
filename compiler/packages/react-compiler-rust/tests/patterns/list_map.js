// Pattern: Array.map for lists
// Tests list rendering with map
function ProductList({ products, onSelect }) {
    const sortedProducts = products.slice().sort((a, b) => a.price - b.price);

    return (
        <ul className="product-list">
            {sortedProducts.map(product => (
                <li key={product.id} onClick={() => onSelect(product)}>
                    <span className="name">{product.name}</span>
                    <span className="price">${product.price}</span>
                    {product.discount > 0 && (
                        <span className="discount">-{product.discount}%</span>
                    )}
                </li>
            ))}
        </ul>
    );
}
