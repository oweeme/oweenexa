import { describe, expect, it, vi } from "vitest";
import { createApi, HttpError } from "../src";

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
