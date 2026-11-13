import { Signal } from "@nexa/reactivity";
import { createScope, disposeScope, effect, onCleanup, runInScope, type Scope } from "./scope";

/**
 * JSX -> DOM real. Sin Virtual DOM y sin re-render de componentes: una
 * función componente corre UNA vez y devuelve nodos reales; lo reactivo
 * se declara con una función (`{() => count.value}`) o pasando una
 * `Signal` directamente (`{count}`) — solo ese nodo de texto/atributo
 * se actualiza cuando cambia.
 */

export type Child = Node | string | number | boolean | null | undefined | Signal<unknown> | (() => Child | Child[]) | Child[];
export type Props = Record<string, unknown> & { children?: Child | Child[] };
export type Component<P = Props> = (props: P) => Node;

export const Fragment = Symbol("nexa.fragment");

const SVG_NS = "http://www.w3.org/2000/svg";
const SVG_TAGS = new Set(["svg", "path", "circle", "rect", "line", "polyline", "polygon", "g", "defs", "use", "text", "ellipse"]);

export function h(tag: string | typeof Fragment | Component<any>, props: Props | null, ...children: Child[]): Node {
    const merged: Props = { ...(props ?? {}) };
    if (children.length > 0) merged.children = children.length === 1 ? children[0] : children;

    if (tag === Fragment) {
        const fragment = document.createDocumentFragment();
        insert(fragment, merged.children);
        return fragment;
    }

    if (typeof tag === "function") {
        // Un componente corre una vez, con su propio ámbito para que
        // sus efectos mueran con el nodo que devolvió, no antes.
        return tag(merged);
    }

    const el = SVG_TAGS.has(tag) ? document.createElementNS(SVG_NS, tag) : document.createElement(tag);
    for (const [name, value] of Object.entries(merged)) {
        if (name === "children") continue;
        applyProp(el, name, value);
    }
    insert(el, merged.children);
    return el;
}

function applyProp(el: Element, name: string, value: unknown): void {
    if (name === "ref" && typeof value === "function") {
        (value as (el: Element) => void)(el);
        return;
    }

    if (name.startsWith("on") && name.length > 2 && typeof value === "function") {
        const type = name.slice(2).toLowerCase();
        el.addEventListener(type, value as EventListener);
        onCleanup(() => el.removeEventListener(type, value as EventListener));
        return;
    }

    // `value={signal}` / `checked={signal}` en un input: enlace en ambos sentidos.
    if ((name === "value" || name === "checked") && value instanceof Signal) {
        const signal = value as Signal<unknown>;
        const prop = name;
        effect(() => {
            (el as any)[prop] = signal.value;
        });
        const event = prop === "checked" ? "change" : "input";
        const listener = () => {
            signal.value = (el as any)[prop];
        };
        el.addEventListener(event, listener);
        onCleanup(() => el.removeEventListener(event, listener));
        return;
    }

    if (value instanceof Signal || (typeof value === "function" && !name.startsWith("on"))) {
        const read = value instanceof Signal ? () => (value as Signal<unknown>).value : (value as () => unknown);
        effect(() => setAttr(el, name, read()));
        return;
    }

    setAttr(el, name, value);
}

function setAttr(el: Element, name: string, value: unknown): void {
    const attr = name === "className" ? "class" : name;

    if (attr === "style" && value && typeof value === "object") {
        Object.assign((el as HTMLElement).style, value);
        return;
    }
    if (value === false || value === null || value === undefined) {
        el.removeAttribute(attr);
        if (attr === "value" || attr === "checked" || attr === "disabled") (el as any)[attr] = attr === "value" ? "" : false;
        return;
    }
    if (attr === "value" || attr === "checked") {
        (el as any)[attr] = value;
        return;
    }
    el.setAttribute(attr, value === true ? "" : String(value));
}

/** Inserta `child` dentro de `parent`, creando efectos para lo reactivo. */
export function insert(parent: Node, child: Child | Child[]): void {
    if (Array.isArray(child)) {
        for (const item of child) insert(parent, item);
        return;
    }
    if (child === null || child === undefined || child === false || child === true) return;

    if (child instanceof Node) {
        parent.appendChild(child);
        return;
    }

    if (child instanceof Signal || typeof child === "function") {
        insertDynamic(parent, child as Signal<unknown> | (() => Child | Child[]));
        return;
    }

    parent.appendChild(document.createTextNode(String(child)));
}

/**
 * Un hueco reactivo: dos marcadores de comentario delimitan la región y
 * cada vez que cambia lo que devuelve la función, se reemplaza SOLO lo
 * que hay entre ellos. El contenido anterior se desmonta (su ámbito se
 * limpia) antes de montar el nuevo.
 */
export function insertDynamic(parent: Node, source: Signal<unknown> | (() => Child | Child[])): void {
    const start = document.createComment("");
    const end = document.createComment("");
    parent.appendChild(start);
    parent.appendChild(end);

    let scope: Scope | null = null;
    let textNode: Text | null = null;

    effect(() => {
        const value = source instanceof Signal ? (source.value as Child | Child[]) : source();

        // Camino rápido y frecuente: texto simple -> reutilizar el mismo nodo.
        if (value === null || value === undefined || typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
            if (scope) {
                disposeScope(scope);
                scope = null;
            }
            const text = value === null || value === undefined || typeof value === "boolean" ? "" : String(value);
            if (textNode && textNode.parentNode === end.parentNode) {
                textNode.data = text;
                return;
            }
            clearBetween(start, end);
            textNode = document.createTextNode(text);
            end.parentNode?.insertBefore(textNode, end);
            return;
        }

        textNode = null;
        if (scope) disposeScope(scope);
        scope = createScope();
        clearBetween(start, end);
        const fragment = document.createDocumentFragment();
        runInScope(scope, () => insert(fragment, value));
        end.parentNode?.insertBefore(fragment, end);
    });

    onCleanup(() => {
        if (scope) disposeScope(scope);
    });
}

function clearBetween(start: Node, end: Node): void {
    let node = start.nextSibling;
    while (node && node !== end) {
        const next = node.nextSibling;
        node.parentNode?.removeChild(node);
        node = next;
    }
}

/** Un hueco reactivo como nodo independiente (lo usan Show/For/Switch). */
export function dynamic(source: Signal<unknown> | (() => Child | Child[])): Node {
    const fragment = document.createDocumentFragment();
    insertDynamic(fragment, source);
    return fragment;
}
