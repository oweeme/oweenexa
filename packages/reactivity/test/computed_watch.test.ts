import { describe, expect, it } from "vitest";
import { computed, effect, state, watch } from "../src";

describe("computed", () => {
    it("has the right value immediately, without waiting for a microtask", () => {
        const count = state(2);
        const doubled = computed(() => count.value * 2);
        expect(doubled.value).toBe(4);
    });

    it("recomputes when a dependency changes", async () => {
        const count = state(2);
        const doubled = computed(() => count.value * 2);

        count.value = 5;
        await Promise.resolve();

        expect(doubled.value).toBe(10);
    });

    it("does not notify subscribers when the recomputed value is identical", async () => {
        const count = state(2);
        const isEven = computed(() => count.value % 2 === 0);
        let runs = 0;

        effect(() => {
            isEven.value;
            runs++;
        });
        expect(runs).toBe(1);

        // 2 -> 4 sigue siendo par: el computed no cambia, así que el
        // efecto que lo lee no debería volver a correr.
        count.value = 4;
        await Promise.resolve();

        expect(runs).toBe(1);
    });

    it("chains: a computed can read another computed", async () => {
        const count = state(1);
        const doubled = computed(() => count.value * 2);
        const quadrupled = computed(() => doubled.value * 2);

        expect(quadrupled.value).toBe(4);

        count.value = 3;
        // Dos saltos de efecto (count -> doubled, doubled -> quadrupled)
        // = dos vueltas de microtask para que se propague del todo —
        // mismo comportamiento que tendría encadenar dos `effect()` a
        // mano, no algo especial de `computed`.
        await Promise.resolve();
        await Promise.resolve();

        expect(quadrupled.value).toBe(12);
    });

    it("peek() reads the current value without subscribing", async () => {
        const count = state(1);
        const doubled = computed(() => count.value * 2);
        let runs = 0;

        effect(() => {
            doubled.peek();
            runs++;
        });
        expect(runs).toBe(1);

        count.value = 2;
        await Promise.resolve();

        expect(runs).toBe(1);
        expect(doubled.peek()).toBe(4);
    });
});

describe("watch", () => {
    it("does NOT call the callback on creation, unlike effect", () => {
        const count = state(0);
        let calls = 0;

        watch(
            () => count.value,
            () => calls++,
        );

        expect(calls).toBe(0);
    });

    it("calls the callback with the new and previous value when the source changes", async () => {
        const count = state(0);
        const seen: Array<[number, number | undefined]> = [];

        watch(
            () => count.value,
            (next, prev) => seen.push([next, prev]),
        );

        count.value = 1;
        await Promise.resolve();

        expect(seen).toEqual([[1, 0]]);
    });

    it("does not call the callback when the source is set to an identical value", async () => {
        const count = state(0);
        let calls = 0;

        watch(
            () => count.value,
            () => calls++,
        );

        count.value = 0;
        await Promise.resolve();

        expect(calls).toBe(0);
    });

    it("stops watching once disposed", async () => {
        const count = state(0);
        let calls = 0;

        const dispose = watch(
            () => count.value,
            () => calls++,
        );
        dispose();

        count.value = 1;
        await Promise.resolve();

        expect(calls).toBe(0);
    });

    it("can watch a computed value", async () => {
        const count = state(1);
        const doubled = computed(() => count.value * 2);
        let calls = 0;

        watch(
            () => doubled.value,
            () => calls++,
        );

        count.value = 2;
        // Mismo salto extra de microtask que al encadenar dos computed
        // (count -> doubled, doubled -> el effect interno de watch).
        await Promise.resolve();
        await Promise.resolve();

        expect(calls).toBe(1);
    });
});
