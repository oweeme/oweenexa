import { describe, expect, it, vi } from "vitest";

const loadStripeMock = vi.fn();

vi.mock("@stripe/stripe-js", () => ({
    loadStripe: (...args: unknown[]) => loadStripeMock(...args),
}));

describe("stripe.load", () => {
    it("delegates to the real loadStripe from @stripe/stripe-js with the given key", async () => {
        const fakeClient = { redirectToCheckout: vi.fn() };
        loadStripeMock.mockResolvedValue(fakeClient);

        const { stripe } = await import("../src/index");
        const client = await stripe.load("pk_test_123");

        expect(loadStripeMock).toHaveBeenCalledWith("pk_test_123");
        expect(client).toBe(fakeClient);
    });
});
