import { effect, mountComponent, state, type Component } from "@nexa/reactivity";

/**
 * Mismo widget que `Dashboard.ts` (Vue 3 real, vía `@nexa/vue-island`),
 * reescrito con el modelo de componentes liviano de `@nexa/reactivity`
 * (Fase 50, issue #19) — la comparación de peso de bundle que pide el
 * propio issue: `Dashboard.ts` + Vue 3 completo pesa ~245KB
 * (`dashboard-island.js`); este archivo, con toda su lógica y sin
 * ningún framework externo, pesa unos pocos KB (ver
 * `docs/FASES-DE-CONSTRUCCION.md`, Fase 50, para el número exacto del
 * bundle real).
 *
 * Dos componentes reales, compuestos: `TaskItem` (hijo) y `TaskList`
 * (padre, que monta N `TaskItem` con `mountComponent`) — props
 * tipadas, sin Virtual DOM: cada uno manipula DOM real una sola vez,
 * la lista completa se reconstruye ante un cambio (mismo criterio que
 * ya usaba `productFilter.island.ts` a mano) — lo nuevo acá es que
 * `TaskItem` es una unidad reutilizable de verdad, no código inline.
 */
interface Task {
    id: number;
    title: string;
    done: boolean;
}

interface TaskItemProps {
    task: Task;
    onToggle: (id: number) => void;
}

const TaskItem: Component<TaskItemProps> = (el, props, ctx) => {
    const li = document.createElement("li");
    li.style.textDecoration = props.task.done ? "line-through" : "none";

    const label = document.createElement("label");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = props.task.done;

    const onChange = () => props.onToggle(props.task.id);
    checkbox.addEventListener("change", onChange);
    ctx.onCleanup(() => checkbox.removeEventListener("change", onChange));

    label.append(checkbox, ` ${props.task.title}`);
    li.appendChild(label);
    el.appendChild(li);
};

interface TaskListProps {
    tasks: Task[];
}

const TaskList: Component<TaskListProps> = (el, props, ctx) => {
    const tasks = state(props.tasks.map((t) => ({ ...t })));

    const heading = document.createElement("h2");
    heading.className = "nx-card-title";
    const list = document.createElement("ul");
    el.append(heading, list);

    const toggle = (id: number) => {
        tasks.value = tasks.value.map((t) => (t.id === id ? { ...t, done: !t.done } : t));
    };

    // Cada `TaskItem` se monta con `mountComponent` (no `ctx.mount`,
    // reservado para composición estática): la lista cambia con cada
    // toggle, así que sus hijos se desmontan y vuelven a montar en
    // cada pasada del `effect`, no solo una vez al montar `TaskList`.
    let disposeItems: Array<() => void> = [];

    const disposeEffect = effect(() => {
        for (const dispose of disposeItems.splice(0)) dispose();

        const pending = tasks.value.filter((t) => !t.done).length;
        heading.textContent = `Pendientes: ${pending}`;
        list.innerHTML = "";

        for (const task of tasks.value) {
            disposeItems.push(mountComponent(TaskItem, list, { task, onToggle: toggle }));
        }
    });

    ctx.onCleanup(() => {
        disposeEffect();
        for (const dispose of disposeItems.splice(0)) dispose();
    });
};

function isTask(value: unknown): value is Task {
    return (
        !!value &&
        typeof value === "object" &&
        typeof (value as Task).id === "number" &&
        typeof (value as Task).title === "string" &&
        typeof (value as Task).done === "boolean"
    );
}

export default function mount(el: Element, props: Record<string, unknown>): () => void {
    const tasks = Array.isArray(props.tasks) ? props.tasks.filter(isTask) : [];
    return mountComponent(TaskList, el, { tasks });
}
