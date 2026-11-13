/** Cliente HTTP propio de Nexa: no depende de Axios ni de ninguna otra librería. */

export class HttpError extends Error {
    constructor(
        public readonly status: number,
        message: string,
    ) {
        super(message || `HTTP ${status}`);
        this.name = "HttpError";
    }
}

export interface HttpClientOptions {
    baseURL?: string;
    /** Inyectable para tests; por defecto, el `fetch` global. */
    fetchImpl?: typeof fetch;
    /**
     * Cabeceras en cada petición — un objeto fijo, o una función que se
     * evalúa en cada llamada (ej. para leer un token que cambia).
     */
    headers?: Record<string, string> | (() => Record<string, string>);
    /**
     * Por defecto `"same-origin"` (lo normal de `fetch`: las cookies van
     * a tu propio origen — que es lo que da el proxy `[proxy]` de
     * `nexa.toml`). Usá `"include"` solo si el backend vive en otro
     * origen *y* responde con CORS + `Access-Control-Allow-Credentials`.
     */
    credentials?: RequestCredentials;
    /**
     * Se llama cuando el backend responde 401 (sesión vencida o ausente)
     * — antes de lanzar el `HttpError`. Ideal para limpiar la sesión y
     * mandar al login. Un 403 es "autenticado pero sin permiso": no
     * dispara esto.
     */
    onUnauthorized?: (error: HttpError) => void;
}

export interface HttpClient {
    get<T>(path: string): Promise<T>;
    post<T>(path: string, body?: unknown): Promise<T>;
    put<T>(path: string, body?: unknown): Promise<T>;
    patch<T>(path: string, body?: unknown): Promise<T>;
    delete<T>(path: string): Promise<T>;
}

/**
 * En el navegador, `/api` es del mismo origen que la app (gracias al
 * `[proxy]` de `nexa.toml`, o a nginx en producción). Dentro de una app
 * de escritorio (Tauri) no hay proxy: la página vive en `tauri://localhost`.
 * `nexa build --target desktop` declara `window.__NEXA_PROXY__ =
 * { "/api": "https://api.miapp.com" }` (de `[desktop.proxy]`) y las rutas
 * que empiezan por ese prefijo se reescriben al backend real — el mismo
 * código de la app funciona en la web y en escritorio.
 */
export function resolveProxiedBase(baseURL: string, map?: Record<string, string>): { url: string; rewritten: boolean } {
    const table = map ?? (globalThis as { __NEXA_PROXY__?: Record<string, string> }).__NEXA_PROXY__;
    if (!table || !baseURL.startsWith("/")) return { url: baseURL, rewritten: false };
    const prefix = Object.keys(table)
        .filter((p) => baseURL === p || baseURL.startsWith(p.replace(/\/$/, "") + "/"))
        .sort((a, b) => b.length - a.length)[0];
    if (prefix === undefined) return { url: baseURL, rewritten: false };
    return { url: table[prefix].replace(/\/$/, "") + baseURL, rewritten: true };
}

export function createApi(options: HttpClientOptions = {}): HttpClient {
    const fetchImpl = options.fetchImpl ?? fetch;

    async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
        const hasBody = body !== undefined;
        const extra = typeof options.headers === "function" ? options.headers() : (options.headers ?? {});
        const headers: Record<string, string> = { ...extra };
        if (hasBody) headers["Content-Type"] = "application/json";

        // Se resuelve en cada llamada: `__NEXA_PROXY__` puede definirse
        // después de crear el cliente.
        const { url: baseURL, rewritten } = resolveProxiedBase(options.baseURL ?? "");
        const response = await fetchImpl(baseURL + path, {
            method,
            headers: Object.keys(headers).length > 0 ? headers : undefined,
            body: hasBody ? JSON.stringify(body) : undefined,
            // Reescrito = otro origen: para que la cookie de sesión viaje
            // hace falta `include` (el backend debe responder con CORS).
            credentials: options.credentials ?? (rewritten ? "include" : undefined),
        });

        if (!response.ok) {
            const error = new HttpError(response.status, await safeText(response));
            if (response.status === 401) options.onUnauthorized?.(error);
            throw error;
        }

        if (response.status === 204) {
            return undefined as T;
        }

        return (await response.json()) as T;
    }

    return {
        get: (path) => request("GET", path),
        post: (path, body) => request("POST", path, body),
        put: (path, body) => request("PUT", path, body),
        patch: (path, body) => request("PATCH", path, body),
        delete: (path) => request("DELETE", path),
    };
}

async function safeText(response: Response): Promise<string> {
    try {
        return await response.text();
    } catch {
        return "";
    }
}
