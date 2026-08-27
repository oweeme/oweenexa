import { defaultFetcher, type PageFetcher } from "./fetcher";

export interface InitRouterOptions {
    root?: Document;
    fetchPage?: PageFetcher;
    /**
     * Compartido con `initPrefetch`: si una ruta ya se precargó, navegar
     * a ella no dispara un segundo fetch.
     */
    cache?: Map<string, string>;
}

/**
 * Intercepta clics en enlaces internos: en vez de dejar que el navegador
 * recargue la página, pide el HTML completo de destino, reemplaza
 * `<title>` y el contenido de `<body>`, y actualiza la URL con
 * `history.pushState`. `popstate` (atrás/adelante) hace lo mismo sin
 * volver a empujar el historial.
 *
 * Nota deliberada: reemplaza *todo* `<body>`, no un fragmento más fino —
 * eso requeriría un contenedor de página estable (`<main id="app">`) que
 * todavía no existe (llega con el sistema de layout, Fase 9). El punto de
 * esta fase es que no haya recarga completa del navegador; refinar qué
 * tanto del DOM se reemplaza es un paso posterior, no un cambio de
 * arquitectura.
 */
export function initRouter(options: InitRouterOptions = {}): () => void {
    const root = options.root ?? document;
    const fetchPage = options.fetchPage ?? defaultFetcher;
    const cache = options.cache ?? new Map<string, string>();

    const onClick = (event: MouseEvent) => {
        if (event.defaultPrevented || event.button !== 0) return;
        if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

        const target = event.target as Element | null;
        const anchor = target?.closest?.("a[href]") as HTMLAnchorElement | null;
        if (!anchor || !isInternalNavigableLink(anchor)) return;

        event.preventDefault();
        void navigate(anchor.href, true);
    };

    const onPopState = () => {
        void navigate(location.href, false);
    };

    async function navigate(url: string, push: boolean): Promise<void> {
        const path = new URL(url, location.href).pathname;
        const html = cache.get(path) ?? (await fetchPage(path));
        cache.delete(path);

        applyPage(root, html);
        if (push) {
            history.pushState({}, "", url);
        }
    }

    root.addEventListener("click", onClick);
    window.addEventListener("popstate", onPopState);

    return () => {
        root.removeEventListener("click", onClick);
        window.removeEventListener("popstate", onPopState);
    };
}

function isInternalNavigableLink(anchor: HTMLAnchorElement): boolean {
    if (anchor.target && anchor.target !== "_self") return false;
    if (anchor.hasAttribute("download")) return false;
    if (anchor.hasAttribute("data-nexa-reload")) return false;

    const url = new URL(anchor.href, location.href);
    return url.origin === location.origin;
}

function applyPage(root: Document, html: string): void {
    const parsed = new DOMParser().parseFromString(html, "text/html");
    root.title = parsed.title;
    root.body.innerHTML = parsed.body.innerHTML;
}
