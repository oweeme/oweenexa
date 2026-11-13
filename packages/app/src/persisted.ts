import { effect, state, type Signal } from "@nexa/reactivity";

/**
 * Una señal que sobrevive a recargar la página — guardada en
 * `localStorage` (por defecto) o `sessionStorage`. Si el almacenamiento
 * no está disponible (modo privado, bloqueado, SSR) se comporta como un
 * `state()` normal: nunca lanza.
 *
 * Ideal para preferencias y para el *estado* de sesión de la UI
 * (`user`, `theme`). Un token secreto real mejor vive en una cookie
 * `HttpOnly` que pone el backend — accesible solo por el navegador, no
 * por JavaScript (ver `[proxy]` en `nexa.toml`).
 */
export function persisted<T>(key: string, initial: T, storage: "local" | "session" = "local"): Signal<T> {
    const store = pickStorage(storage);
    let start = initial;
    try {
        const raw = store?.getItem(key);
        if (raw !== null && raw !== undefined) start = JSON.parse(raw) as T;
    } catch {
        /* JSON inválido o storage bloqueado: se usa `initial` */
    }

    const signal = state<T>(start);
    effect(() => {
        const value = signal.value;
        try {
            if (value === undefined) store?.removeItem(key);
            else store?.setItem(key, JSON.stringify(value));
        } catch {
            /* cuota llena o storage bloqueado: la señal sigue funcionando en memoria */
        }
    });
    return signal;
}

function pickStorage(kind: "local" | "session"): Storage | null {
    try {
        return kind === "local" ? globalThis.localStorage : globalThis.sessionStorage;
    } catch {
        return null;
    }
}
