import { describe, expect, it, vi } from "vitest";
import { mutation } from "../src";

describe("mutation", () => {
    it("no se dispara sola: solo cuando se llama a execute()", () => {
        const execute = vi.fn().mockResolvedValue({ ok: true });
        const m = mutation({ execute });

        expect(execute).not.toHaveBeenCalled();
        expect(m.loading.value).toBe(false);
    });

    it("pone loading=true durante execute() y lo baja al terminar", async () => {
        const execute = vi.fn().mockResolvedValue({ ok: true });
        const m = mutation({ execute });

        const promise = m.execute();
        expect(m.loading.value).toBe(true);

        await promise;
        expect(m.loading.value).toBe(false);
        expect(m.data.value).toEqual({ ok: true });
    });

    it("pasa los argumentos a la función real, y también los devuelve/relanza en error", async () => {
        const execute = vi.fn().mockRejectedValue(new Error("stock agotado"));
        const m = mutation<[productId: number], never>({ execute });

        await expect(m.execute(42)).rejects.toThrow("stock agotado");

        expect(execute).toHaveBeenCalledWith(42);
        expect(m.loading.value).toBe(false);
        expect(m.error.value).toBeInstanceOf(Error);
    });
});
