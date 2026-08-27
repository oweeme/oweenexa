import { defaultFetcher, type PageFetcher } from "./fetcher";

export interface InitPrefetchOptions {
    root?: Document;
    fetchPage?: PageFetcher;
    cache?: Map<string, string>;
    /** ms de hover antes de precargar — evita disparar en un simple "paso de largo". */
    hoverDelay?: number;
}

/**
 * Prefetch básico (Fase 6): al pasar el cursor sobre un enlace interno
 * (con un pequeño retraso) o al tocarlo en pantalla táctil, pide su HTML
 * por adelantado y lo deja en `cache` — la misma que usa `initRouter`,
 * para que la navegación real no vuelva a pedirlo por red.
 */
export function initPrefetch(options: InitPrefetchOptions = {}): () => void {
    const root = options.root ?? document;
    const fetchPage = options.fetchPage ?? defaultFetcher;
    const cache = options.cache ?? new Map<string, string>();
    const hoverDelay = options.hoverDelay ?? 60;

    const timers = new WeakMap<Element, ReturnType<typeof setTimeout>>();

    function schedule(anchor: HTMLAnchorElement): void {
        if (timers.has(anchor)) return;
        const timer = setTimeout(() => {
            timers.delete(anchor);
            void prefetch(anchor);
        }, hoverDelay);
        timers.set(anchor, timer);
    }

    function cancel(anchor: HTMLAnchorElement): void {
        const timer = timers.get(anchor);
        if (timer !== undefined) {
            clearTimeout(timer);
            timers.delete(anchor);
        }
    }

    async function prefetch(anchor: HTMLAnchorElement): Promise<void> {
        const url = new URL(anchor.href, location.href);
        if (url.origin !== location.origin || cache.has(url.pathname)) return;

        try {
            cache.set(url.pathname, await fetchPage(url.pathname));
        } catch {
            // Un prefetch fallido no debe romper nada: la navegación real
            // hará su propio fetch (y podrá fallar de forma visible ahí).
        }
    }

    const anchorFrom = (event: Event) =>
        (event.target as Element | null)?.closest?.("a[href]") as HTMLAnchorElement | null;

    const onPointerOver = (event: Event) => {
        const anchor = anchorFrom(event);
        if (anchor) schedule(anchor);
    };
    const onPointerOut = (event: Event) => {
        const anchor = anchorFrom(event);
        if (anchor) cancel(anchor);
    };
    const onTouchStart = (event: Event) => {
        const anchor = anchorFrom(event);
        if (anchor) void prefetch(anchor);
    };

    root.addEventListener("pointerover", onPointerOver);
    root.addEventListener("pointerout", onPointerOut);
    root.addEventListener("touchstart", onTouchStart, { passive: true });

    return () => {
        root.removeEventListener("pointerover", onPointerOver);
        root.removeEventListener("pointerout", onPointerOut);
        root.removeEventListener("touchstart", onTouchStart);
    };
}
