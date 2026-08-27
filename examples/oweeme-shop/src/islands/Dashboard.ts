import { computed, defineComponent, h, ref } from "vue";

/**
 * Un componente Vue 3 real y de verdad (Fase 16) — reactividad de Vue
 * (`ref`/`computed`), no de `@nexa/reactivity`. Representa el tipo de
 * panel administrativo (tipo SDLC/Trello) que no tiene sentido como
 * contenido SEO: por eso vive detrás de una isla, en vez de ser una
 * página Nexa normal.
 *
 * Sin `<template>` a propósito: el bundling de este ejemplo usa esbuild
 * puro (igual que `packages/stripe` -> `public/vendor/nexa-stripe.js`),
 * que no trae un compilador de SFC de Vue. Un proyecto real añadiría
 * `@vitejs/plugin-vue` (o equivalente) a su propio pipeline de build —
 * eso es una decisión de tooling del proyecto, no algo que Nexa imponga.
 */
interface Task {
    id: number;
    title: string;
    done: boolean;
}

export const Dashboard = defineComponent({
    props: {
        tasks: { type: Array as () => Task[], default: () => [] },
    },
    setup(props) {
        const tasks = ref<Task[]>(props.tasks.map((t) => ({ ...t })));
        const pending = computed(() => tasks.value.filter((t) => !t.done).length);

        const toggle = (id: number) => {
            const task = tasks.value.find((t) => t.id === id);
            if (task) task.done = !task.done;
        };

        return () =>
            h("div", { class: "nx-card" }, [
                h("h2", { class: "nx-card-title" }, `Pendientes: ${pending.value}`),
                h(
                    "ul",
                    tasks.value.map((task) =>
                        h(
                            "li",
                            { key: task.id, style: { textDecoration: task.done ? "line-through" : "none" } },
                            [
                                h("label", [
                                    h("input", {
                                        type: "checkbox",
                                        checked: task.done,
                                        onChange: () => toggle(task.id),
                                    }),
                                    ` ${task.title}`,
                                ]),
                            ],
                        ),
                    ),
                ),
            ]);
    },
});
