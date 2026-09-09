/**
 * Acceso directo a la Cache API, independiente de `[pwa.cache]`
 * (Fase 18) — ese mecanismo vive dentro del service worker que genera
 * `nexa add pwa` y responde a `fetch` real; esto es para cachear algo
 * puntual desde código de página/isla normal, sin necesitar `[pwa]`
 * activado en absoluto. Envoltorio delgado sobre la Cache API nativa —
 * mismo criterio que `platform.storage()`: sin protocolo propio, solo
 * un `open()` inyectable para tests y un error explícito si el entorno
 * no la tiene.
 */
export interface CacheStore {
    match(request: string | Request): Promise<Response | undefined>;
    put(request: string | Request, response: Response): Promise<void>;
    delete(request: string | Request): Promise<boolean>;
}

export async function openCache(name: string, backend?: CacheStorage): Promise<CacheStore> {
    const storage = backend ?? (typeof caches !== "undefined" ? caches : undefined);
    if (!storage) {
        throw new Error("[nexa/platform] la Cache API no está disponible en este entorno.");
    }

    const cache = await storage.open(name);
    return {
        match: (request) => cache.match(request),
        put: (request, response) => cache.put(request, response),
        delete: (request) => cache.delete(request),
    };
}
