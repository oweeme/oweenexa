import { describe, expect, it } from "vitest";

import { createSessionStorage, createStorage } from "../src/storage";

function fakeStorage(): Storage {
    const data = new Map<string, string>();
    return {
        getItem: (key: string) => data.get(key) ?? null,
        setItem: (key: string, value: string) => void data.set(key, value),
        removeItem: (key: string) => void data.delete(key),
        clear: () => data.clear(),
        key: () => null,
        get length() {
            return data.size;
        },
    };
}

describe("createStorage", () => {
    it("sets and reads back a value", () => {
        const storage = createStorage(fakeStorage());
        storage.set("locale", "es");
        expect(storage.get("locale")).toBe("es");
    });

    it("returns null for a key that was never set", () => {
        const storage = createStorage(fakeStorage());
        expect(storage.get("missing")).toBeNull();
    });

    it("removes a key", () => {
        const storage = createStorage(fakeStorage());
        storage.set("locale", "es");
        storage.remove("locale");
        expect(storage.get("locale")).toBeNull();
    });

    it("defaults to the real localStorage when no backend is given", () => {
        const storage = createStorage();
        storage.set("nexa-platform-test", "1");
        expect(localStorage.getItem("nexa-platform-test")).toBe("1");
        storage.remove("nexa-platform-test");
    });
});

describe("createSessionStorage", () => {
    it("sets and reads back a value", () => {
        const storage = createSessionStorage(fakeStorage());
        storage.set("draft", "wizard-step-2");
        expect(storage.get("draft")).toBe("wizard-step-2");
    });

    it("returns null for a key that was never set", () => {
        const storage = createSessionStorage(fakeStorage());
        expect(storage.get("missing")).toBeNull();
    });

    it("removes a key", () => {
        const storage = createSessionStorage(fakeStorage());
        storage.set("draft", "x");
        storage.remove("draft");
        expect(storage.get("draft")).toBeNull();
    });

    it("defaults to the real sessionStorage when no backend is given", () => {
        const storage = createSessionStorage();
        storage.set("nexa-platform-session-test", "1");
        expect(sessionStorage.getItem("nexa-platform-session-test")).toBe("1");
        storage.remove("nexa-platform-session-test");
    });

    it("is independent from createStorage — same key, distinto almacenamiento real", () => {
        const local = createStorage(fakeStorage());
        const session = createSessionStorage(fakeStorage());
        local.set("shared-key", "from-local");
        session.set("shared-key", "from-session");
        expect(local.get("shared-key")).toBe("from-local");
        expect(session.get("shared-key")).toBe("from-session");
    });
});
