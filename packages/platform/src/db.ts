/**
 * Wrapper mínimo sobre IndexedDB — no un ORM, un `get`/`set`/`delete`/
 * `list` basado en promesas por "colección" (un object store), en la
 * misma línea que `platform.storage()`. La API nativa de IndexedDB es
 * verbosa y basada en callbacks/eventos; esto es lo mínimo para no
 * tener que reinventarlo en cada proyecto.
 *
 * Cada colección es su propia base de datos IndexedDB
 * (`nexa-db-<nombre>`), con un único object store fijo adentro — evita
 * el problema de tener que declarar de antemano todos los nombres de
 * colección que un proyecto va a usar (IndexedDB exige conocer los
 * object stores al crear/subir de versión la base, algo que no tiene
 * sentido pedirle a `platform.db("loQueSea")` llamado dinámicamente).
 *
 * Tauri/Capacitor: sin rama nativa distinta — igual que
 * `platform.storage()`, el `IndexedDB` del propio webview alcanza; no
 * hace falta (ni existe todavía) un plugin nativo para esto.
 */

const STORE_NAME = "items";

export interface Collection<T = unknown> {
    get(id: string): Promise<T | undefined>;
    set(id: string, value: T): Promise<void>;
    delete(id: string): Promise<void>;
    list(): Promise<T[]>;
}

function openDatabase(name: string, factory: IDBFactory): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        const request = factory.open(`nexa-db-${name}`, 1);
        request.onupgradeneeded = () => {
            request.result.createObjectStore(STORE_NAME);
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}

function runRequest<T>(request: IDBRequest<T>): Promise<T> {
    return new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
}

/**
 * Sincrónico a propósito (`platform.db("x").set(...)`, sin `await`
 * antes de `.db()`) — cada método individual abre la conexión real de
 * forma diferida y devuelve su propia promesa.
 */
export function openCollection<T = unknown>(name: string, backend?: IDBFactory): Collection<T> {
    const factory = backend ?? (typeof indexedDB !== "undefined" ? indexedDB : undefined);
    if (!factory) {
        throw new Error("[nexa/platform] IndexedDB no está disponible en este entorno.");
    }

    const dbPromise = openDatabase(name, factory);

    async function withStore<R>(mode: IDBTransactionMode, run: (store: IDBObjectStore) => IDBRequest<R>): Promise<R> {
        const db = await dbPromise;
        const tx = db.transaction(STORE_NAME, mode);
        return runRequest(run(tx.objectStore(STORE_NAME)));
    }

    return {
        get: (id) => withStore("readonly", (store) => store.get(id)),
        set: async (id, value) => {
            await withStore("readwrite", (store) => store.put(value, id));
        },
        delete: async (id) => {
            await withStore("readwrite", (store) => store.delete(id));
        },
        list: () => withStore("readonly", (store) => store.getAll()),
    };
}
