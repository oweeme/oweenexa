import { describe, expect, it } from "vitest";

import { expectEntry, getEntry, type Manifest } from "../src/manifest";

function manifest(): Manifest {
    return {
        "3": { event: "click", handler: "buy", module: "/assets/Home-3.js", strategy: "interaction" },
    };
}

describe("getEntry", () => {
    it("returns the entry for a known node id", () => {
        expect(getEntry(manifest(), 3).handler).toBe("buy");
    });

    it("accepts a string id too", () => {
        expect(getEntry(manifest(), "3").handler).toBe("buy");
    });

    it("throws listing the known ids when the node id is missing", () => {
        expect(() => getEntry(manifest(), 99)).toThrow(/99/);
        expect(() => getEntry(manifest(), 99)).toThrow(/3/);
    });
});

describe("expectEntry", () => {
    it("does not throw when every expected field matches", () => {
        expect(() => expectEntry(manifest(), 3, { event: "click", strategy: "interaction" })).not.toThrow();
    });

    it("only checks the fields that were passed", () => {
        expect(() => expectEntry(manifest(), 3, { handler: "buy" })).not.toThrow();
    });

    it("throws naming the mismatched field and both values", () => {
        expect(() => expectEntry(manifest(), 3, { strategy: "load" })).toThrow(/strategy/);
        expect(() => expectEntry(manifest(), 3, { strategy: "load" })).toThrow(/"load"/);
        expect(() => expectEntry(manifest(), 3, { strategy: "load" })).toThrow(/"interaction"/);
    });
});
