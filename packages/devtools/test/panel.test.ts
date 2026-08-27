import { describe, expect, it } from "vitest";

import { createPanelContainer, render } from "../src/panel";
import type { Diagnostics } from "../src/diagnostics";

function diagnostics(overrides: Partial<Diagnostics> = {}): Diagnostics {
    return {
        pattern: "/",
        classification: { static: 3, dynamic: 1, interactive: 1, async: 0, total: 5 },
        initialJsBytes: 6144,
        activation: [{ id: "3", event: "click", handler: "buy", module: "/assets/Home-3.js", strategy: "interaction" }],
        seoWarnings: [],
        pkgWarnings: [],
        ...overrides,
    };
}

describe("createPanelContainer", () => {
    it("appends a single fixed-position container to the body", () => {
        const container = createPanelContainer(document);

        expect(container.isConnected).toBe(true);
        expect(container.getAttribute("data-nexa-devtools")).toBe("1");
        expect(container.style.position).toBe("fixed");
    });
});

describe("render", () => {
    it("shows the node classification counts and initial JS size", () => {
        const container = createPanelContainer(document);
        render(container, diagnostics());

        expect(container.textContent).toContain("5 nodos");
        expect(container.textContent).toContain("6.0kb");
        expect(container.textContent).toContain("static 3");
    });

    it("lists activation entries with their strategy", () => {
        const container = createPanelContainer(document);
        render(container, diagnostics());

        expect(container.textContent).toContain("buy");
        expect(container.textContent).toContain("interaction");
    });

    it("shows a warning count in the collapsed summary when there are warnings", () => {
        const container = createPanelContainer(document);
        render(container, diagnostics({ seoWarnings: [{ code: "NEXA-SEO-001", message: "falta title" }] }));

        expect(container.textContent).toContain("1 aviso(s)");
        expect(container.textContent).toContain("NEXA-SEO-001");
    });

    it("does not mention warnings in the summary when there are none", () => {
        const container = createPanelContainer(document);
        render(container, diagnostics());

        expect(container.textContent).not.toContain("aviso(s)");
    });

    it("escapes warning messages instead of injecting raw HTML", () => {
        const container = createPanelContainer(document);
        render(container, diagnostics({ seoWarnings: [{ code: "X", message: "<img src=x onerror=alert(1)>" }] }));

        expect(container.querySelector("img")).toBeNull();
        expect(container.innerHTML).toContain("&lt;img");
    });
});
