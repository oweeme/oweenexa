import { defineConfig } from "vitest/config";

export default defineConfig({
    test: {
        environment: "happy-dom",
        // Node >= 25 trae un `localStorage` global propio (inutilizable sin
        // `--localstorage-file`) que tapa al de happy-dom.
        execArgv: ["--no-experimental-webstorage"],
    },
});
