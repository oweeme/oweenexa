import { describe, expect, it } from "vitest";

import { openCache } from "../src/cache";

function fakeCacheStorage(): CacheStorage {
    const stores = new Map<string, Map<string, Response>>();
    return {
        open: async (name: string) => {
            if (!stores.has(name)) stores.set(name, new Map());
            const store = stores.get(name)!;
            return {
                match: async (request: RequestInfo | URL) => store.get(String(request)),
                put: async (request: RequestInfo | URL, response: Response) => {
                    store.set(String(request), response);
                },
                delete: async (request: RequestInfo | URL) => store.delete(String(request)),
            } as unknown as Cache;
        },
    } as unknown as CacheStorage;
}

describe("openCache", () => {
    it("put() guarda una respuesta y match() la devuelve", async () => {
        const cache = await openCache("api-cache", fakeCacheStorage());
        const response = new Response(JSON.stringify({ ok: true }));

        await cache.put("/api/data", response);
        const cached = await cache.match("/api/data");

        expect(cached).toBe(response);
    });

    it("match() de una clave nunca guardada devuelve undefined", async () => {
        const cache = await openCache("api-cache", fakeCacheStorage());
        expect(await cache.match("/nunca-existio")).toBeUndefined();
    });

    it("delete() elimina la entrada", async () => {
        const cache = await openCache("api-cache", fakeCacheStorage());
        await cache.put("/api/data", new Response("x"));

        const deleted = await cache.delete("/api/data");
        expect(deleted).toBe(true);
        expect(await cache.match("/api/data")).toBeUndefined();
    });

    it("dos cachés con nombre distinto no comparten entradas", async () => {
        const backend = fakeCacheStorage();
        const a = await openCache("cache-a", backend);
        const b = await openCache("cache-b", backend);

        await a.put("/x", new Response("desde-a"));
        expect(await b.match("/x")).toBeUndefined();
    });

    it("tira un error explícito si la Cache API no está disponible en este entorno", async () => {
        await expect(openCache("api-cache", undefined as unknown as CacheStorage)).rejects.toThrow(
            /Cache API no está disponible/,
        );
    });
});
