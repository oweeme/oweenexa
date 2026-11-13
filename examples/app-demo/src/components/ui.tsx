export function Tag({ text }) {
    return <span class="tag">{text}</span>;
}

export function Button({ onPress, children }) {
    return (
        <button class="nx-btn nx-btn-primary" onClick={onPress}>
            {children}
        </button>
    );
}
