import { describe, expect, it } from "vitest";
import { effect, state } from "../src";

describe("state + effect", () => {
    it("runs the effect immediately on creation", () => {
        const count = state(0);
        let seen: number | undefined;

        effect(() => {
            seen = count.value;
        });

        expect(seen).toBe(0);
    });

    it("reruns when a read signal changes", async () => {
        const count = state(0);
        let runs = 0;

        effect(() => {
            count.value;
            runs++;
        });
        expect(runs).toBe(1);

        count.value = 1;
        await Promise.resolve();

        expect(runs).toBe(2);
    });

    it("does not rerun when an unrelated signal changes", async () => {
        const a = state(0);
        const b = state(0);
        let runs = 0;

        effect(() => {
            a.value;
            runs++;
        });
        expect(runs).toBe(1);

        b.value = 42;
        await Promise.resolve();

        expect(runs).toBe(1);
    });

    it("batches several synchronous writes into a single rerun", async () => {
        const count = state(0);
        let runs = 0;

        effect(() => {
            count.value;
            runs++;
        });
        expect(runs).toBe(1);

        count.value = 1;
        count.value = 2;
        count.value = 3;
        await Promise.resolve();

        expect(runs).toBe(2);
        expect(count.value).toBe(3);
    });

    it("dispose() stops future reruns", async () => {
        const count = state(0);
        let runs = 0;

        const dispose = effect(() => {
            count.value;
            runs++;
        });
        dispose();

        count.value = 1;
        await Promise.resolve();

        expect(runs).toBe(1);
    });

    it("peek() reads the value without creating a dependency", async () => {
        const count = state(0);
        let runs = 0;

        effect(() => {
            count.peek();
            runs++;
        });
        expect(runs).toBe(1);

        count.value = 1;
        await Promise.resolve();

        expect(runs).toBe(1);
    });

    it("setting the same value does not trigger a rerun", async () => {
        const count = state(0);
        let runs = 0;

        effect(() => {
            count.value;
            runs++;
        });
        expect(runs).toBe(1);

        count.value = 0;
        await Promise.resolve();

        expect(runs).toBe(1);
    });
});
