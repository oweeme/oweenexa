import { afterEach, describe, expect, it, vi } from "vitest";
import { activateManually, idle, interaction, load, manual, visible } from "../src/strategies";

function makeEl(): HTMLElement {
    const el = document.createElement("button");
    document.body.appendChild(el);
    return el;
}

describe("strategies", () => {
    it("interaction: solo dispara tras el primer evento, no antes", async () => {
        const el = makeEl();
        const trigger = vi.fn().mockResolvedValue(undefined);

        interaction(el, "click", trigger);
        expect(trigger).not.toHaveBeenCalled();

        el.dispatchEvent(new Event("click"));
        await Promise.resolve();
        expect(trigger).toHaveBeenCalledTimes(1);
    });

    it("interaction: reproduce el evento original una vez activado", async () => {
        const el = makeEl();
        let resolveTrigger: () => void = () => {};
        const trigger = vi.fn(() => new Promise<void>((resolve) => (resolveTrigger = resolve)));

        const replayed = vi.fn();
        el.addEventListener("click", replayed);

        interaction(el, "click", trigger);
        el.dispatchEvent(new Event("click", { bubbles: true }));

        // El propio dispatch de arriba ya cuenta como una llamada a `replayed`
        // (el listener estaba puesto antes de que `interaction` quitara el
        // suyo), así que confirmamos que se reproduce una segunda vez tras
        // resolver el trigger.
        expect(replayed).toHaveBeenCalledTimes(1);

        resolveTrigger();
        await vi.waitFor(() => expect(replayed).toHaveBeenCalledTimes(2));
    });

    it("load: dispara de inmediato", () => {
        const el = makeEl();
        const trigger = vi.fn().mockResolvedValue(undefined);
        load(el, "click", trigger);
        expect(trigger).toHaveBeenCalledTimes(1);
    });

    it("manual: no dispara sola; activateManually sí la dispara", async () => {
        const el = makeEl();
        const trigger = vi.fn().mockResolvedValue(undefined);

        manual(el, "click", trigger);
        expect(trigger).not.toHaveBeenCalled();

        await activateManually(el);
        expect(trigger).toHaveBeenCalledTimes(1);
    });

    it("manual: activar un elemento sin registrar no falla", async () => {
        const el = makeEl();
        await expect(activateManually(el)).resolves.toBeUndefined();
    });

    describe("visible", () => {
        const OriginalIntersectionObserver = globalThis.IntersectionObserver;
        let capturedCallback: IntersectionObserverCallback | undefined;
        let disconnect: ReturnType<typeof vi.fn>;

        afterEach(() => {
            globalThis.IntersectionObserver = OriginalIntersectionObserver;
        });

        function stubIntersectionObserver() {
            disconnect = vi.fn();
            class FakeIntersectionObserver {
                constructor(callback: IntersectionObserverCallback) {
                    capturedCallback = callback;
                }
                observe = vi.fn();
                disconnect = disconnect;
                unobserve = vi.fn();
                takeRecords = vi.fn(() => []);
            }
            // @ts-expect-error -- stub deliberadamente incompleto para el test
            globalThis.IntersectionObserver = FakeIntersectionObserver;
        }

        it("dispara al entrar en el viewport, y se desconecta", () => {
            stubIntersectionObserver();
            const el = makeEl();
            const trigger = vi.fn().mockResolvedValue(undefined);

            visible(el, "click", trigger);
            expect(trigger).not.toHaveBeenCalled();

            capturedCallback?.(
                [{ isIntersecting: true } as IntersectionObserverEntry],
                {} as IntersectionObserver,
            );

            expect(trigger).toHaveBeenCalledTimes(1);
            expect(disconnect).toHaveBeenCalledTimes(1);
        });
    });

    describe("idle", () => {
        const original = globalThis.requestIdleCallback;

        afterEach(() => {
            globalThis.requestIdleCallback = original;
        });

        it("dispara cuando el navegador está inactivo", () => {
            let capturedCallback: IdleRequestCallback | undefined;
            globalThis.requestIdleCallback = ((cb: IdleRequestCallback) => {
                capturedCallback = cb;
                return 1;
            }) as typeof requestIdleCallback;

            const el = makeEl();
            const trigger = vi.fn().mockResolvedValue(undefined);

            idle(el, "click", trigger);
            expect(trigger).not.toHaveBeenCalled();

            capturedCallback?.({ didTimeout: false, timeRemaining: () => 0 });
            expect(trigger).toHaveBeenCalledTimes(1);
        });
    });
});
