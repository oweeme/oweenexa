export function Tag({ text }: { text: string }) {
    return <span class="tag">{text}</span>;
}

export function Button({ onPress, children }: { onPress: () => void; children?: unknown }) {
    return (
        <button class="nx-btn nx-btn-primary" onClick={onPress}>
            {children}
        </button>
    );
}
