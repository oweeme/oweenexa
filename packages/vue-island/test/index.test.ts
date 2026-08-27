import { describe, expect, it } from "vitest";
import { defineComponent, h, nextTick, ref } from "vue";
import { defineVueIsland } from "../src";

// Un componente Vue real, sin `<template>` (el build runtime-only de
// `vue` que usan los bundlers no trae el compilador de plantillas) —
// pero sigue siendo Vue de verdad: `ref`, reactividad, ciclo de vida.
const Counter = defineComponent({
    props: { initial: { type: Number, default: 0 } },
    setup(props) {
        const count = ref(props.initial);
        return () => h("button", { onClick: () => count.value++ }, `count: ${count.value}`);
    },
});

describe("defineVueIsland — Fase 16, criterio de salida", () => {
    it("monta un componente Vue real sobre el elemento, con las props recibidas", () => {
        const el = document.createElement("div");
        document.body.appendChild(el);

        const mount = defineVueIsland(Counter);
        mount(el, { initial: 5 });

        expect(el.textContent).toBe("count: 5");
    });

    it("el componente montado es interactivo de verdad (estado reactivo de Vue, no de Nexa)", async () => {
        const el = document.createElement("div");
        document.body.appendChild(el);

        defineVueIsland(Counter)(el, { initial: 0 });

        el.querySelector("button")!.dispatchEvent(new MouseEvent("click", { bubbles: true }));
        await nextTick();

        expect(el.textContent).toBe("count: 1");
    });

    it("el cleanup devuelto por mount() desmonta la app de Vue", () => {
        const el = document.createElement("div");
        document.body.appendChild(el);

        const unmount = defineVueIsland(Counter)(el, {});
        expect(el.textContent).not.toBe("");

        unmount();
        expect(el.innerHTML).toBe("");
    });
});
