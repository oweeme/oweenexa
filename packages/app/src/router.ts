import { Signal } from "@nexa/reactivity";
import type { Component } from "./h";
import { createScope, disposeScope, runInScope, type Scope } from "./scope";

export interface AppRoute {
    /** Patrón relativo a `base`, ej. `/users/:id`. */
    pattern: string;
    /** `import()` lazy del módulo de la página — un chunk por ruta. */
    load: () => Promise<{ default: Component<PageProps> }>;
}

export interface PageProps extends Record<string, unknown> {
    params: Record<string, string>;
    query: URLSearchParams;
}

/** Devuelve `false` para cancelar, o una ruta (string) para redirigir. */
export type Guard = (to: { path: string; params: Record<string, string>; query: URLSearchParams }) => boolean | string | void | Promise<boolean | string | void>;

export interface StartAppOptions {
    routes: AppRoute[];
    /** Elemento donde se monta la app (por defecto `#app`). */
    root?: Element | string;
    /** Prefijo bajo el cual vive la app (por defecto `/app`). */
    base?: string;
    notFound?: Component<PageProps>;
    /** Se ejecuta antes de cada navegación — la base de rutas protegidas. */
    guard?: Guard;
    /** Se muestra mientras carga el chunk de una ruta. */
    loading?: () => Node;
}

export interface RouterState {
    path: Signal<string>;
    params: Signal<Record<string, string>>;
    query: Signal<URLSearchParams>;
}

let active: { navigate: (to: string, replace?: boolean) => Promise<void>; state: RouterState; base: string } | null = null;

/** Estado reactivo de la ruta actual — `route.path.value`, etc. */
export const route: RouterState = {
    path: new Signal("/"),
    params: new Signal<Record<string, string>>({}),
    query: new Signal(new URLSearchParams()),
};

export function navigate(to: string, options: { replace?: boolean } = {}): Promise<void> {
    if (!active) throw new Error("[nexa/app] navigate() antes de startApp()");
    return active.navigate(to, options.replace);
}

export function matchPattern(pattern: string, path: string): Record<string, string> | null {
    const p = pattern.split("/").filter(Boolean);
    const s = path.split("/").filter(Boolean);
    const params: Record<string, string> = {};
    for (let i = 0; i < p.length; i++) {
        const seg = p[i];
        if (seg.startsWith("*")) {
            params[seg.slice(1) || "rest"] = s.slice(i).map(decodeURIComponent).join("/");
            return params;
        }
        if (i >= s.length) return null;
        if (seg.startsWith(":")) params[seg.slice(1)] = decodeURIComponent(s[i]);
        else if (seg !== s[i]) return null;
    }
    return p.length === s.length ? params : null;
}

/** Más específica primero: estáticas antes que dinámicas antes que comodín. */
function rank(pattern: string): number {
    return pattern
        .split("/")
        .filter(Boolean)
        .reduce((n, seg) => n + (seg.startsWith("*") ? 0 : seg.startsWith(":") ? 2 : 3), 1);
}

export function startApp(options: StartAppOptions): () => void {
    const base = (options.base ?? "/app").replace(/\/$/, "");
    const rootEl =
        typeof options.root === "string" ? document.querySelector(options.root) : (options.root ?? document.querySelector("#app"));
    if (!rootEl) throw new Error("[nexa/app] no se encontró el elemento raíz de la app");
    const root: Element = rootEl;

    const routes = [...options.routes].sort((a, b) => rank(b.pattern) - rank(a.pattern));
    let pageScope: Scope | null = null;
    let token = 0;

    // En una app de escritorio el webview puede abrir `/app/index.html`.
    const strip = (full: string) => {
        const rel = full.startsWith(base) ? full.slice(base.length) || "/" : full;
        return rel === "/index.html" ? "/" : rel;
    };

    async function render(fullPath: string, search: string): Promise<string | null> {
        const path = strip(fullPath);
        const query = new URLSearchParams(search);
        let params: Record<string, string> = {};
        let matched: AppRoute | null = null;
        for (const r of routes) {
            const m = matchPattern(r.pattern, path);
            if (m) {
                matched = r;
                params = m;
                break;
            }
        }

        if (options.guard) {
            const verdict = await options.guard({ path, params, query });
            if (typeof verdict === "string") return verdict;
            if (verdict === false) return null;
        }

        const mine = ++token;
        let component: Component<PageProps>;
        if (matched) {
            if (options.loading && !pageScope) {
                root.replaceChildren(options.loading());
            }
            component = (await matched.load()).default;
        } else if (options.notFound) {
            component = options.notFound;
        } else {
            component = () => {
                const el = document.createElement("p");
                el.textContent = `404 — no hay una ruta para ${path}`;
                return el;
            };
        }
        // Una navegación más nueva ganó mientras se cargaba el chunk.
        if (mine !== token) return null;

        if (pageScope) disposeScope(pageScope);
        pageScope = createScope();
        route.path.value = path;
        route.params.value = params;
        route.query.value = query;

        const node = runInScope(pageScope, () => component({ params, query }));
        root.replaceChildren(node);
        return null;
    }

    async function go(to: string, replace = false): Promise<void> {
        const url = new URL(to, location.origin);
        const full = url.pathname.startsWith(base) ? url.pathname : base + (url.pathname === "/" ? "" : url.pathname);
        const redirect = await render(full, url.search);
        if (redirect !== null) return go(redirect, true);
        const target = full + url.search;
        if (replace) history.replaceState({}, "", target);
        else if (target !== location.pathname + location.search) history.pushState({}, "", target);
        window.scrollTo(0, 0);
    }

    const onClick = (event: MouseEvent) => {
        if (event.defaultPrevented || event.button !== 0) return;
        if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
        const anchor = (event.target as Element | null)?.closest?.("a[href]") as HTMLAnchorElement | null;
        if (!anchor || (anchor.target && anchor.target !== "_self")) return;
        if (anchor.hasAttribute("download") || anchor.hasAttribute("data-nexa-reload")) return;
        const url = new URL(anchor.href, location.href);
        // Solo lo que vive dentro de la app; el resto (el sitio estático) es una carga normal.
        if (url.origin !== location.origin) return;
        if (url.pathname !== base && !url.pathname.startsWith(base + "/")) return;
        event.preventDefault();
        void go(url.pathname + url.search);
    };
    const onPop = () => {
        void render(location.pathname, location.search);
    };

    document.addEventListener("click", onClick);
    window.addEventListener("popstate", onPop);

    active = { navigate: go, state: route, base };
    void go(location.pathname + location.search, true);

    return () => {
        document.removeEventListener("click", onClick);
        window.removeEventListener("popstate", onPop);
        if (pageScope) disposeScope(pageScope);
        active = null;
    };
}
