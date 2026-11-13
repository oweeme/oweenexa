import { describe, expect, it, beforeEach, vi } from "vitest";
import { h, Fragment, Show, For, Switch, Match, state, startApp, matchPattern, route, navigate, effect, createScope, disposeScope, runInScope } from "../src/index";

const tick = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
    document.body.innerHTML = '<div id="app"></div>';
    history.replaceState({}, "", "/app");
});

describe("h()", () => {
    it("crea DOM real con atributos, eventos y texto", () => {
        let clicked = 0;
        const el = h("button", { class: "x", onClick: () => clicked++ }, "hola") as HTMLElement;
        expect(el.outerHTML).toBe('<button class="x">hola</button>');
        el.click();
        expect(clicked).toBe(1);
    });

    it("un hijo función es reactivo y solo cambia ese nodo de texto", async () => {
        const n = state(0);
        const p = h("p", null, "n=", () => n.value) as HTMLElement;
        const textBefore = p.childNodes[1];
        expect(p.textContent).toBe("n=0");
        n.value = 5;
        await tick();
        expect(p.textContent).toBe("n=5");
        // el mismo nodo de texto se reutilizó (sin recrear el DOM)
        expect(Array.from(p.childNodes).includes(textBefore)).toBe(true);
    });

    it("una Signal como hijo y como atributo", async () => {
        const n = state("a");
        const el = h("span", { title: n }, n) as HTMLElement;
        expect(el.getAttribute("title")).toBe("a");
        n.value = "b";
        await tick();
        expect(el.getAttribute("title")).toBe("b");
        expect(el.textContent).toBe("b");
    });

    it("value={signal} enlaza en ambos sentidos", async () => {
        const v = state("x");
        const input = h("input", { value: v }) as HTMLInputElement;
        expect(input.value).toBe("x");
        input.value = "nuevo";
        input.dispatchEvent(new Event("input"));
        expect(v.value).toBe("nuevo");
        v.value = "otra";
        await tick();
        expect(input.value).toBe("otra");
    });

    it("los componentes son funciones con props y children", () => {
        const Card = (p: { title: string; children?: any }) => h("div", { class: "card" }, h("h2", null, p.title), p.children);
        const el = h(Card as any, { title: "T" }, h("p", null, "cuerpo")) as HTMLElement;
        expect(el.innerHTML).toBe("<h2>T</h2><p>cuerpo</p>");
    });

    it("Fragment devuelve varios nodos", () => {
        const f = h(Fragment, null, h("i", null, "1"), h("b", null, "2"));
        const host = document.createElement("div");
        host.appendChild(f);
        expect(host.innerHTML).toBe("<i>1</i><b>2</b>");
    });
});

describe("control de flujo", () => {
    it("Show alterna y desmonta los efectos del contenido que sale", async () => {
        const on = state(true);
        let runs = 0;
        const inner = state(0);
        const host = h("div", null, Show({ when: on, fallback: "off", children: () => { return h("span", null, () => { runs++; return inner.value; }); } } as any)) as HTMLElement;
        // children como función dentro de Show se evalúa como hijo reactivo
        expect(host.textContent).toContain("0");
        on.value = false;
        await tick();
        expect(host.textContent).toBe("off");
        const before = runs;
        inner.value = 9;
        await tick();
        expect(runs).toBe(before); // el efecto de lo desmontado ya no corre
    });

    it("For conserva los nodos de los items que siguen y reordena", async () => {
        const list = state(["a", "b", "c"]);
        const ul = h("ul", null, For({ each: list, children: (item) => h("li", null, item) })) as HTMLElement;
        const lis = () => Array.from(ul.querySelectorAll("li"));
        expect(lis().map((l) => l.textContent)).toEqual(["a", "b", "c"]);
        const bNode = lis()[1];
        list.value = ["c", "b", "d"];
        await tick();
        expect(lis().map((l) => l.textContent)).toEqual(["c", "b", "d"]);
        expect(lis()[1]).toBe(bNode);
    });

    it("For muestra el fallback si la lista está vacía", async () => {
        const list = state<string[]>([]);
        const ul = h("ul", null, For({ each: list, fallback: "vacío", children: (i) => h("li", null, i) })) as HTMLElement;
        expect(ul.textContent).toBe("vacío");
        list.value = ["x"];
        await tick();
        expect(ul.textContent).toBe("x");
    });

    it("Switch/Match elige el primer caso verdadero", async () => {
        const s = state("loading");
        const el = h("div", null, Switch({ fallback: "?", children: [
            Match({ when: () => s.value === "loading", children: "cargando" }),
            Match({ when: () => s.value === "ok", children: "listo" }),
        ] })) as HTMLElement;
        expect(el.textContent).toBe("cargando");
        s.value = "ok";
        await tick();
        expect(el.textContent).toBe("listo");
        s.value = "x";
        await tick();
        expect(el.textContent).toBe("?");
    });
});

describe("ámbitos", () => {
    it("disposeScope detiene los efectos creados dentro", async () => {
        const n = state(0);
        let runs = 0;
        const scope = createScope();
        runInScope(scope, () => effect(() => { n.value; runs++; }));
        n.value = 1;
        await tick();
        expect(runs).toBe(2);
        disposeScope(scope);
        n.value = 2;
        await tick();
        expect(runs).toBe(2);
    });
});

describe("router", () => {
    it("matchPattern: estático, dinámico, comodín", () => {
        expect(matchPattern("/users/:id", "/users/7")).toEqual({ id: "7" });
        expect(matchPattern("/users/:id", "/users")).toBeNull();
        expect(matchPattern("/", "/")).toEqual({});
        expect(matchPattern("/docs/*rest", "/docs/a/b")).toEqual({ rest: "a/b" });
    });

    it("renderiza la ruta, navega con enlaces sin recargar, y pasa params", async () => {
        const Home = () => h("div", null, h("a", { href: "/app/users/42" }, "ir"));
        const User = (p: any) => h("h1", null, "user ", p.params.id);
        const stop = startApp({
            routes: [
                { pattern: "/", load: async () => ({ default: Home as any }) },
                { pattern: "/users/:id", load: async () => ({ default: User as any }) },
            ],
        });
        await tick();
        expect(document.querySelector("#app")!.innerHTML).toContain("ir");
        (document.querySelector("a") as HTMLAnchorElement).click();
        await tick(); await tick();
        expect(document.querySelector("#app")!.textContent).toBe("user 42");
        expect(location.pathname).toBe("/app/users/42");
        expect(route.params.value).toEqual({ id: "42" });
        stop();
    });

    it("guard puede redirigir (rutas protegidas)", async () => {
        const Login = () => h("p", null, "login");
        const Secret = () => h("p", null, "secreto");
        const stop = startApp({
            routes: [
                { pattern: "/login", load: async () => ({ default: Login as any }) },
                { pattern: "/secret", load: async () => ({ default: Secret as any }) },
            ],
            guard: (to) => (to.path === "/secret" ? "/login" : undefined),
        });
        await tick();
        await navigate("/app/secret");
        await tick();
        expect(document.querySelector("#app")!.textContent).toBe("login");
        expect(location.pathname).toBe("/app/login");
        stop();
    });

    it("la página anterior se desmonta (sus efectos mueren) al navegar", async () => {
        const n = state(0);
        let runs = 0;
        const A = () => h("p", null, () => { runs++; return n.value; });
        const B = () => h("p", null, "b");
        const stop = startApp({
            routes: [
                { pattern: "/", load: async () => ({ default: A as any }) },
                { pattern: "/b", load: async () => ({ default: B as any }) },
            ],
        });
        await tick();
        await navigate("/app/b");
        await tick();
        const before = runs;
        n.value = 1;
        await tick();
        expect(runs).toBe(before);
        stop();
    });

    it("404 por defecto", async () => {
        const stop = startApp({ routes: [] });
        await tick();
        await navigate("/app/nada");
        await tick();
        expect(document.querySelector("#app")!.textContent).toContain("404");
        stop();
    });
});

import { persisted, createApi } from "../src/index";

function memoryStorage(): Storage {
    const m = new Map<string, string>();
    return {
        getItem: (k) => (m.has(k) ? m.get(k)! : null),
        setItem: (k, v) => void m.set(k, String(v)),
        removeItem: (k) => void m.delete(k),
        clear: () => m.clear(),
        key: (i) => Array.from(m.keys())[i] ?? null,
        get length() { return m.size; },
    } as Storage;
}

describe("persisted()", () => {
    // Node reciente trae su propio `localStorage` global (sin archivo, inutilizable):
    // en un navegador real es el de la página; en el test, uno en memoria.
    beforeEach(() => vi.stubGlobal("localStorage", memoryStorage()));

    it("lee el valor guardado al crearse", () => {
        localStorage.setItem("k1", JSON.stringify({ name: "Ana" }));
        expect(persisted("k1", { name: "?" }).value).toEqual({ name: "Ana" });
    });

    it("usa `initial` si no hay nada o el JSON está roto", () => {
        expect(persisted("nada", 5).value).toBe(5);
        localStorage.setItem("roto", "{no es json");
        expect(persisted("roto", "ok").value).toBe("ok");
    });

    it("guarda cada cambio, y undefined borra la clave", async () => {
        const s = persisted<string | undefined>("k2", "a");
        s.value = "b";
        await tick();
        expect(localStorage.getItem("k2")).toBe('"b"');
        s.value = undefined;
        await tick();
        expect(localStorage.getItem("k2")).toBeNull();
    });
});

describe("capa de datos re-exportada", () => {
    it("createApi está disponible desde @nexa/app", () => {
        expect(typeof createApi).toBe("function");
    });
});
