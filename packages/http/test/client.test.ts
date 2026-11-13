import { describe, expect, it, vi } from "vitest";
import { createApi, HttpError, resolveProxiedBase } from "../src";

function fakeFetch(status: number, body: unknown): typeof fetch {
    return vi.fn(async () =>
        new Response(status === 204 ? null : JSON.stringify(body), {
            status,
            headers: { "Content-Type": "application/json" },
        }),
    ) as unknown as typeof fetch;
}

describe("createApi", () => {
    it("GET devuelve el JSON de la respuesta", async () => {
        const fetchImpl = fakeFetch(200, { id: 1, name: "iPhone 17" });
        const api = createApi({ baseURL: "https://api.example.com", fetchImpl });

        const product = await api.get<{ id: number; name: string }>("/products/1");

        expect(product).toEqual({ id: 1, name: "iPhone 17" });
        expect(fetchImpl).toHaveBeenCalledWith(
            "https://api.example.com/products/1",
            expect.objectContaining({ method: "GET" }),
        );
    });

    it("POST serializa el body como JSON y pone el Content-Type", async () => {
        const fetchImpl = fakeFetch(200, { ok: true });
        const api = createApi({ fetchImpl });

        await api.post("/cart", { productId: 1 });

        const [, init] = (fetchImpl as ReturnType<typeof vi.fn>).mock.calls[0] as [string, RequestInit];
        expect(init.method).toBe("POST");
        expect(init.body).toBe(JSON.stringify({ productId: 1 }));
        expect((init.headers as Record<string, string>)["Content-Type"]).toBe("application/json");
    });

    it("una respuesta no-ok lanza HttpError con el status real", async () => {
        const fetchImpl = fakeFetch(404, { message: "not found" });
        const api = createApi({ fetchImpl });

        await expect(api.get("/products/999")).rejects.toMatchObject({
            name: "HttpError",
            status: 404,
        });
    });

    it("204 No Content devuelve undefined en vez de intentar parsear JSON", async () => {
        const fetchImpl = fakeFetch(204, undefined);
        const api = createApi({ fetchImpl });

        await expect(api.delete("/products/1")).resolves.toBeUndefined();
    });

    it("HttpError es una instancia real de Error", () => {
        const err = new HttpError(500, "boom");
        expect(err).toBeInstanceOf(Error);
        expect(err.status).toBe(500);
        expect(err.message).toBe("boom");
    });
});

describe("createApi — autenticación (Fase 61)", () => {
    it("envía cabeceras fijas y calculadas en cada llamada", async () => {
        const fetchImpl = fakeFetch(200, {});
        let token = "a";
        const api = createApi({ fetchImpl, headers: () => ({ Authorization: `Bearer ${token}` }) });
        await api.get("/x");
        token = "b";
        await api.post("/x", { n: 1 });
        const calls = (fetchImpl as any).mock.calls;
        expect(calls[0][1].headers).toEqual({ Authorization: "Bearer a" });
        expect(calls[1][1].headers).toEqual({ Authorization: "Bearer b", "Content-Type": "application/json" });
    });

    it("pasa `credentials` a fetch", async () => {
        const fetchImpl = fakeFetch(200, {});
        await createApi({ fetchImpl, credentials: "include" }).get("/x");
        expect((fetchImpl as any).mock.calls[0][1].credentials).toBe("include");
    });

    it("un 401 llama a onUnauthorized y además lanza HttpError", async () => {
        const onUnauthorized = vi.fn();
        const api = createApi({ fetchImpl: fakeFetch(401, "no"), onUnauthorized });
        await expect(api.get("/x")).rejects.toBeInstanceOf(HttpError);
        expect(onUnauthorized).toHaveBeenCalledTimes(1);
        expect(onUnauthorized.mock.calls[0][0].status).toBe(401);
    });

    it("un 403 NO dispara onUnauthorized (autenticado pero sin permiso)", async () => {
        const onUnauthorized = vi.fn();
        const api = createApi({ fetchImpl: fakeFetch(403, "no"), onUnauthorized });
        await expect(api.get("/x")).rejects.toMatchObject({ status: 403 });
        expect(onUnauthorized).not.toHaveBeenCalled();
    });

    it("patch usa el método PATCH", async () => {
        const fetchImpl = fakeFetch(200, { ok: true });
        await createApi({ fetchImpl }).patch("/x", { a: 1 });
        expect((fetchImpl as any).mock.calls[0][1].method).toBe("PATCH");
    });
});

describe("resolveProxiedBase / escritorio (Fase 63)", () => {
    const map = { "/api": "https://api.miapp.com/", "/api/v2": "https://v2.miapp.com" };

    it("sin tabla (navegador normal) no toca nada", () => {
        expect(resolveProxiedBase("/api", undefined)).toEqual({ url: "/api", rewritten: false });
    });

    it("reescribe el prefijo al backend real, sin doble barra", () => {
        expect(resolveProxiedBase("/api", map)).toEqual({ url: "https://api.miapp.com/api", rewritten: true });
        expect(resolveProxiedBase("/api/users", map).url).toBe("https://api.miapp.com/api/users");
    });

    it("gana el prefijo más largo, y solo por segmento completo", () => {
        expect(resolveProxiedBase("/api/v2/x", map).url).toBe("https://v2.miapp.com/api/v2/x");
        expect(resolveProxiedBase("/apix", map).rewritten).toBe(false);
    });

    it("una URL absoluta o relativa sin barra inicial no se toca", () => {
        expect(resolveProxiedBase("https://otro.com/api", map).rewritten).toBe(false);
        expect(resolveProxiedBase("api", map).rewritten).toBe(false);
    });

    it("createApi usa __NEXA_PROXY__ y manda credentials:include al reescribir", async () => {
        (globalThis as any).__NEXA_PROXY__ = { "/api": "http://127.0.0.1:9100" };
        try {
            const fetchImpl = fakeFetch(200, {});
            await createApi({ baseURL: "/api", fetchImpl }).get("/users");
            const [url, init] = (fetchImpl as any).mock.calls[0];
            expect(url).toBe("http://127.0.0.1:9100/api/users");
            expect(init.credentials).toBe("include");
        } finally {
            delete (globalThis as any).__NEXA_PROXY__;
        }
    });
});
