import { describe, expect, it, vi } from "vitest";

import { effect, mountComponent, state } from "../src";
import type { Component } from "../src";

describe("mountComponent", () => {
    it("calls the component with the container and the given props", () => {
        const spy = vi.fn();
        const component: Component<{ label: string }> = spy;
        const container = document.createElement("div");

        mountComponent(component, container, { label: "hola" });

        expect(spy).toHaveBeenCalledOnce();
        expect(spy.mock.calls[0][0]).toBe(container);
        expect(spy.mock.calls[0][1]).toEqual({ label: "hola" });
    });

    it("runs onCleanup functions, in reverse order, when disposed", () => {
        const order: number[] = [];
        const component: Component<undefined> = (_el, _props, ctx) => {
            ctx.onCleanup(() => order.push(1));
            ctx.onCleanup(() => order.push(2));
        };

        const dispose = mountComponent(component, document.createElement("div"), undefined);
        expect(order).toEqual([]);
        dispose();
        expect(order).toEqual([2, 1]);
    });

    it("disposing twice only runs cleanup once", () => {
        const cleanup = vi.fn();
        const component: Component<undefined> = (_el, _props, ctx) => ctx.onCleanup(cleanup);

        const dispose = mountComponent(component, document.createElement("div"), undefined);
        dispose();
        dispose();

        expect(cleanup).toHaveBeenCalledTimes(1);
    });

    describe("composición: un componente monta a otro", () => {
        interface ItemProps {
            label: string;
        }

        const Item: Component<ItemProps> = (el, props) => {
            el.textContent = props.label;
        };

        interface ListProps {
            labels: string[];
        }

        const List: Component<ListProps> = (el, props, ctx) => {
            for (const label of props.labels) {
                const itemEl = document.createElement("li");
                el.appendChild(itemEl);
                ctx.mount(Item, itemEl, { label });
            }
        };

        it("un componente padre renderiza N hijos con props tipadas", () => {
            const container = document.createElement("ul");
            mountComponent(List, container, { labels: ["a", "b", "c"] });

            expect(container.children.length).toBe(3);
            expect(Array.from(container.children).map((li) => li.textContent)).toEqual(["a", "b", "c"]);
        });

        it("desmontar el padre desmonta también a los hijos", () => {
            const childCleanup = vi.fn();
            const Child: Component<undefined> = (_el, _props, ctx) => ctx.onCleanup(childCleanup);
            const Parent: Component<undefined> = (el, _props, ctx) => {
                const childEl = document.createElement("div");
                el.appendChild(childEl);
                ctx.mount(Child, childEl, undefined);
            };

            const dispose = mountComponent(Parent, document.createElement("div"), undefined);
            dispose();

            expect(childCleanup).toHaveBeenCalledOnce();
        });
    });

    it("una Signal pasada como prop permite que el hijo reaccione a cambios del padre, sin ningún mecanismo nuevo", () => {
        const count = state(0);
        const Child: Component<{ count: typeof count }> = (el, props, ctx) => {
            const disposeEffect = effect(() => {
                el.textContent = String(props.count.value);
            });
            ctx.onCleanup(disposeEffect);
        };

        const container = document.createElement("div");
        mountComponent(Child, container, { count });
        expect(container.textContent).toBe("0");

        count.value = 5;
        return new Promise<void>((resolve) => {
            queueMicrotask(() => {
                expect(container.textContent).toBe("5");
                resolve();
            });
        });
    });
});
