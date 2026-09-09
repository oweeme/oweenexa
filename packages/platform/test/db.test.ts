import { describe, expect, it } from "vitest";
import { IDBFactory } from "fake-indexeddb";

import { openCollection } from "../src/db";

// Una base de datos IndexedDB real y en memoria (no un mock hecho a
// mano) — `fake-indexeddb` implementa el mismo motor asíncrono y
// transaccional que un navegador real, no una aproximación. A
// propósito NO se usa `fake-indexeddb/auto` (que inyectaría un
// `indexedDB` global): cada test pasa su propia instancia explícita,
// así que el entorno de test se queda sin ningún `indexedDB` global —
// necesario para que el último test ("no disponible") sea real.
function freshFactory(): IDBFactory {
    return new IDBFactory();
}

describe("openCollection", () => {
    it("set() guarda un valor y get() lo devuelve", async () => {
        const users = openCollection<{ name: string }>("users", freshFactory());
        await users.set("1", { name: "Ada" });
        expect(await users.get("1")).toEqual({ name: "Ada" });
    });

    it("get() de un id nunca guardado devuelve undefined", async () => {
        const users = openCollection("users", freshFactory());
        expect(await users.get("nunca-existio")).toBeUndefined();
    });

    it("delete() elimina la entrada", async () => {
        const users = openCollection("users", freshFactory());
        await users.set("1", { name: "Ada" });
        await users.delete("1");
        expect(await users.get("1")).toBeUndefined();
    });

    it("list() devuelve todos los valores guardados", async () => {
        const users = openCollection<{ name: string }>("users", freshFactory());
        await users.set("1", { name: "Ada" });
        await users.set("2", { name: "Linus" });

        const all = await users.list();
        expect(all).toHaveLength(2);
        expect(all).toEqual(expect.arrayContaining([{ name: "Ada" }, { name: "Linus" }]));
    });

    it("set() sobre un id existente lo reemplaza, no lo duplica", async () => {
        const users = openCollection<{ name: string }>("users", freshFactory());
        await users.set("1", { name: "Ada" });
        await users.set("1", { name: "Ada Lovelace" });

        expect(await users.get("1")).toEqual({ name: "Ada Lovelace" });
        expect(await users.list()).toHaveLength(1);
    });

    it("dos colecciones con nombre distinto no comparten datos", async () => {
        const factory = freshFactory();
        const users = openCollection<{ name: string }>("users", factory);
        const products = openCollection<{ name: string }>("products", factory);

        await users.set("1", { name: "Ada" });
        expect(await products.get("1")).toBeUndefined();
    });

    it("tira un error explícito si IndexedDB no está disponible en este entorno", () => {
        expect(() => openCollection("users", undefined as unknown as IDBFactory)).toThrow(
            /IndexedDB no está disponible/,
        );
    });
});
