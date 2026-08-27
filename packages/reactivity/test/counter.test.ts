import { describe, expect, it } from "vitest";
import { bindText, state } from "../src";

describe("Fase 4 — criterio de salida", () => {
    it("un contador actualiza solo su TextNode, sin tocar el resto del DOM", async () => {
        const count = state(0);

        // <button>{count}</button>
        const button = document.createElement("button");
        const textNode = document.createTextNode(String(count.value));
        button.appendChild(textNode);

        // Un hermano no relacionado: nunca debería cambiar.
        const sibling = document.createElement("p");
        const siblingText = document.createTextNode("no debería cambiar");
        sibling.appendChild(siblingText);

        // Instrumentamos `textNode.data` para contar cada escritura real.
        let writes = 0;
        let raw = textNode.data;
        Object.defineProperty(textNode, "data", {
            configurable: true,
            get: () => raw,
            set: (value: string) => {
                writes++;
                raw = value;
            },
        });

        bindText(textNode, () => count.value); // escritura inicial: pone "0" (ya lo era)
        writes = 0; // solo nos interesan las escrituras causadas por el incremento

        function increment() {
            count.value++;
        }

        increment();
        await Promise.resolve(); // el scheduler agrupa por microtask

        expect(textNode.data).toBe("1");
        expect(writes).toBe(1);
        expect(siblingText.data).toBe("no debería cambiar");
        // El <button> no se reconstruyó: sigue teniendo el mismo único hijo.
        expect(button.childNodes.length).toBe(1);
        expect(button.childNodes[0]).toBe(textNode);
    });
});
