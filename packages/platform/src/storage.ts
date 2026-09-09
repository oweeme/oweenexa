/**
 * Almacenamiento clave/valor. A propósito no hay una rama nativa
 * distinta por plataforma: `localStorage`/`sessionStorage` funcionan
 * igual dentro del webview de Tauri y de Capacitor que en un navegador
 * normal — no hace falta (ni existe todavía) un plugin nativo para esto
 * tan básico. Persistencia que sobreviva un `reset` del webview
 * (Capacitor `Preferences`, un plugin de `fs` en Tauri) queda fuera de
 * esta fase.
 */
export interface KeyValueStorage {
    get(key: string): string | null;
    set(key: string, value: string): void;
    remove(key: string): void;
}

function wrapStorage(store: Storage): KeyValueStorage {
    return {
        get: (key) => store.getItem(key),
        set: (key, value) => store.setItem(key, value),
        remove: (key) => store.removeItem(key),
    };
}

export function createStorage(backend?: Storage): KeyValueStorage {
    const store = backend ?? (typeof localStorage !== "undefined" ? localStorage : undefined);
    if (!store) {
        throw new Error("[nexa/platform] no hay almacenamiento disponible en este entorno.");
    }
    return wrapStorage(store);
}

/**
 * Mismo contrato que `createStorage()`, pero sobre `sessionStorage` —
 * vive mientras dure la pestaña, no entre sesiones. Útil para un paso
 * de wizard o un filtro temporal que no debería sobrevivir a cerrar el
 * navegador (a diferencia de `localStorage`, que si sobrevive).
 */
export function createSessionStorage(backend?: Storage): KeyValueStorage {
    const store = backend ?? (typeof sessionStorage !== "undefined" ? sessionStorage : undefined);
    if (!store) {
        throw new Error("[nexa/platform] no hay almacenamiento de sesión disponible en este entorno.");
    }
    return wrapStorage(store);
}
