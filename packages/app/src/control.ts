import { Signal } from "@nexa/reactivity";
import { dynamic, Fragment as Fragment_, h, type Child } from "./h";
import { createScope, disposeScope, effect, onCleanup, runInScope, type Scope } from "./scope";

type Reader<T> = T | Signal<T> | (() => T);

function read<T>(value: Reader<T>): T {
    if (value instanceof Signal) return value.value as T;
    if (typeof value === "function") return (value as () => T)();
    return value;
}

/** `<Show when={() => user.value} fallback={<Login/>}>...</Show>` */
export function Show(props: { when: Reader<unknown>; fallback?: Child; children?: Child | Child[] }): Node {
    return dynamic(() => (read(props.when) ? (props.children ?? null) : (props.fallback ?? null)));
}

/**
 * `<For each={() => items.value}>{(item, index) => <li>{item.name}</li>}</For>`
 *
 * Reconcilia por identidad de elemento: un item que sigue en la lista
 * conserva su nodo (y su estado); solo se crean los nuevos y se
 * desmontan los que salieron. El orden se corrige moviendo nodos, no
 * recreándolos.
 */
export function For<T>(props: {
    each: Reader<readonly T[] | null | undefined>;
    children: (item: T, index: () => number) => Node;
    fallback?: Child;
}): Node {
    const fragment = document.createDocumentFragment();
    const start = document.createComment("for");
    const end = document.createComment("/for");
    fragment.appendChild(start);
    fragment.appendChild(end);

    interface Entry {
        node: Node[];
        scope: Scope;
        index: Signal<number>;
    }
    let entries = new Map<T, Entry[]>();
    let fallbackScope: Scope | null = null;
    let fallbackNodes: Node[] = [];

    const clearFallback = () => {
        if (fallbackScope) disposeScope(fallbackScope);
        fallbackScope = null;
        for (const node of fallbackNodes) node.parentNode?.removeChild(node);
        fallbackNodes = [];
    };

    effect(() => {
        const list = read(props.each) ?? [];
        const parent = end.parentNode as Node;
        const next = new Map<T, Entry[]>();
        const ordered: Entry[] = [];

        list.forEach((item, i) => {
            const pool = entries.get(item);
            let entry = pool?.shift();
            if (entry) {
                entry.index.value = i;
            } else {
                const scope = createScope();
                const index = new Signal(i);
                const node = runInScope(scope, () => props.children(item, () => index.value));
                const nodes = node instanceof DocumentFragment ? Array.from(node.childNodes) : [node];
                entry = { node: nodes, scope, index };
                // Un nodo recién creado debe estar en el DOM para poder moverlo después.
                const holder = document.createDocumentFragment();
                holder.appendChild(node);
                entry.node = Array.from(holder.childNodes);
                for (const n of entry.node) parent.insertBefore(n, end);
            }
            const bucket = next.get(item) ?? [];
            bucket.push(entry);
            next.set(item, bucket);
            ordered.push(entry);
        });

        // Lo que ya no está: desmontar.
        for (const pool of entries.values()) {
            for (const gone of pool) {
                disposeScope(gone.scope);
                for (const n of gone.node) n.parentNode?.removeChild(n);
            }
        }
        entries = next;

        // Orden: recorrer de atrás hacia adelante moviendo solo si hace falta.
        let cursor: Node = end;
        for (let i = ordered.length - 1; i >= 0; i--) {
            const nodes = ordered[i].node;
            for (let j = nodes.length - 1; j >= 0; j--) {
                if (nodes[j].nextSibling !== cursor) parent.insertBefore(nodes[j], cursor);
                cursor = nodes[j];
            }
        }

        if (ordered.length === 0 && props.fallback !== undefined && fallbackNodes.length === 0) {
            fallbackScope = createScope();
            const holder = document.createDocumentFragment();
            runInScope(fallbackScope, () => {
                const node = h(Fragment_, null, props.fallback as Child);
                holder.appendChild(node);
            });
            fallbackNodes = Array.from(holder.childNodes);
            for (const n of fallbackNodes) parent.insertBefore(n, end);
        } else if (ordered.length > 0) {
            clearFallback();
        }
    });

    onCleanup(() => {
        for (const pool of entries.values()) for (const e of pool) disposeScope(e.scope);
        clearFallback();
    });

    return fragment;
}


/** `<Switch fallback={...}><Match when={...}>...</Match>...</Switch>` */
export interface MatchProps {
    when: Reader<unknown>;
    children?: Child | Child[];
}
export function Match(props: MatchProps): Node {
    // Un Match solo describe un caso; Switch lo lee, nunca se monta solo.
    const marker = document.createComment("match");
    (marker as any).__nexaMatch = props;
    return marker;
}

export function Switch(props: { fallback?: Child; children?: Child | Child[] }): Node {
    const cases: MatchProps[] = [];
    const raw = Array.isArray(props.children) ? props.children : [props.children];
    for (const child of raw.flat() as unknown[]) {
        const match = child && (child as any).__nexaMatch;
        if (match) cases.push(match as MatchProps);
    }
    return dynamic(() => {
        for (const c of cases) if (read(c.when)) return c.children ?? null;
        return props.fallback ?? null;
    });
}
