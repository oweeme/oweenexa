import { effect as rawEffect } from "@nexa/reactivity";

/**
 * Ámbito de limpieza: todo `effect()`/`onCleanup()` creado mientras un
 * ámbito está activo se desmonta junto con él. Es lo que permite que
 * una página de Modo App se destruya entera (efectos, listeners,
 * timers) al navegar a otra, sin que el usuario lleve la cuenta.
 */
export interface Scope {
    cleanups: Array<() => void>;
    disposed: boolean;
}

let current: Scope | null = null;

export function createScope(): Scope {
    return { cleanups: [], disposed: false };
}

/** Ejecuta `fn` con `scope` activo y devuelve su resultado. */
export function runInScope<T>(scope: Scope, fn: () => T): T {
    const previous = current;
    current = scope;
    try {
        return fn();
    } finally {
        current = previous;
    }
}

export function disposeScope(scope: Scope): void {
    if (scope.disposed) return;
    scope.disposed = true;
    for (const cleanup of scope.cleanups.splice(0).reverse()) {
        try {
            cleanup();
        } catch (error) {
            console.error("[nexa/app] error en una limpieza:", error);
        }
    }
}

/** Registra `fn` para cuando se desmonte el ámbito actual (si no hay uno, no hace nada). */
export function onCleanup(fn: () => void): void {
    current?.cleanups.push(fn);
}

/** Como `effect` de `@nexa/reactivity`, pero atado al ámbito activo. */
export function effect(fn: () => void): () => void {
    const dispose = rawEffect(fn);
    onCleanup(dispose);
    return dispose;
}

export function currentScope(): Scope | null {
    return current;
}
