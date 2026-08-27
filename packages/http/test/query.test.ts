import { describe, expect, it, vi } from "vitest";
import { query } from "../src";

function deferred<T>() {
    let resolve!: (value: T) => void;
    let reject!: (err: unknown) => void;
    const promise = new Promise<T>((res, rej) => {
        resolve = res;
        reject = rej;
    });
    return { promise, resolve, reject };
}

describe("query — cache-first", () => {
    it("empieza en loading y resuelve data cuando termina el fetch", async () => {
        const { promise, resolve } = deferred<{ id: number }>();
        const fetch = vi.fn(() => promise);

        const q = query({ key: "product-1", fetch, cache: new Map() });
        expect(q.loading.value).toBe(true);
        expect(q.data.value).toBeUndefined();

        resolve({ id: 1 });
        await promise;
        await Promise.resolve();

        expect(q.loading.value).toBe(false);
        expect(q.data.value).toEqual({ id: 1 });
    });

    it("con la misma key en la misma caché, no vuelve a pedir por red", async () => {
        const cache = new Map();
        const fetch = vi.fn().mockResolvedValue({ id: 1 });

        const first = query({ key: "product-1", fetch, cache });
        await vi.waitFor(() => expect(first.loading.value).toBe(false));
        expect(fetch).toHaveBeenCalledTimes(1);

        const second = query({ key: "product-1", fetch, cache });
        expect(fetch).toHaveBeenCalledTimes(1); // no un segundo fetch
        expect(second.loading.value).toBe(false);
        expect(second.data.value).toEqual({ id: 1 });
    });

    it("refetch() vuelve a pedir aunque ya haya caché", async () => {
        const cache = new Map([["product-1", { data: { id: 1 } }]]);
        const fetch = vi.fn().mockResolvedValue({ id: 2 });

        const q = query({ key: "product-1", fetch, cache });
        expect(fetch).not.toHaveBeenCalled(); // cache-first: no dispara sola

        await q.refetch();
        expect(q.data.value).toEqual({ id: 2 });
    });
});

describe("query — network-first", () => {
    it("espera a la red antes de mostrar nada, incluso con caché disponible", async () => {
        const cache = new Map([["product-1", { data: { id: 1, stale: true } }]]);
        const { promise, resolve } = deferred<{ id: number }>();
        const fetch = vi.fn(() => promise);

        const q = query({ key: "product-1", fetch, cachePolicy: "network-first", cache });
        expect(q.loading.value).toBe(true);
        expect(q.data.value).toBeUndefined();

        resolve({ id: 2 });
        await promise;
        await Promise.resolve();

        expect(q.data.value).toEqual({ id: 2 });
    });

    it("si la red falla, cae al último dato bueno en caché", async () => {
        const cache = new Map([["product-1", { data: { id: 1 } }]]);
        const fetch = vi.fn().mockRejectedValue(new Error("network down"));

        const q = query({ key: "product-1", fetch, cachePolicy: "network-first", cache });
        await vi.waitFor(() => expect(q.loading.value).toBe(false));

        expect(q.data.value).toEqual({ id: 1 });
        expect(q.error.value).toBeInstanceOf(Error);
    });
});

describe("query — stale-while-revalidate", () => {
    it("muestra la caché de inmediato (sin loading) y revalida en segundo plano", async () => {
        const cache = new Map([["product-1", { data: { id: 1, stale: true } }]]);
        const { promise, resolve } = deferred<{ id: number }>();
        const fetch = vi.fn(() => promise);

        const q = query({ key: "product-1", fetch, cachePolicy: "stale-while-revalidate", cache });

        expect(q.loading.value).toBe(false);
        expect(q.data.value).toEqual({ id: 1, stale: true });
        expect(fetch).toHaveBeenCalledTimes(1); // sí revalida, aunque no lo muestre como "loading"

        resolve({ id: 1, stale: false });
        await promise;
        await Promise.resolve();

        expect(q.data.value).toEqual({ id: 1, stale: false });
    });
});
