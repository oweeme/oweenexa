import { updateActiveLinks } from "./active-links";
import { defaultFetcher, type PageFetcher } from "./fetcher";

const SLOT_SELECTOR = "[data-nexa-slot]";
const MANIFEST_SELECTOR = "script[data-nexa-manifest]";

export interface InitRouterOptions {
    root?: Document;
    fetchPage?: PageFetcher;
    /**
     * Compartido con `initPrefetch`: si una ruta ya se precargó, navegar
     * a ella no dispara un segundo fetch.
     */
    cache?: Map<string, string>;
    /**
     * Se llama después de aplicar la página de destino — con el
     * contenedor real que cambió (Fase 32): el elemento `data-nexa-slot`
     * si el proyecto usa `src/layout.tsx`, o el propio `document` si no.
     * Es lo que le permite a quien arma el bootstrap (`nexa-cli::bootstrap`)
     * reactivar exactamente lo que cambió — su manifiesto de eventos, sus
     * formularios, sus islas — sin tocar lo que vive fuera de ese
     * contenedor (el header/nav del layout, con cualquier isla que haya
     * montado, sigue en pie sin remount). Sin esto, el contenido nuevo
     * queda con el HTML correcto pero sin nada de JS activado — un
     * `<script>` insertado vía `innerHTML` nunca se ejecuta solo, es
     * comportamiento estándar del navegador, no un bug de Nexa. Bug real
     * encontrado con un navegador real: un botón en la página de destino
     * de una navegación SPA no respondía a clics en absoluto.
     */
    onNavigate?: (root: ParentNode) => void;
}

/**
 * Intercepta clics en enlaces internos: en vez de dejar que el navegador
 * recargue la página, pide el HTML completo de destino, reemplaza
 * `<title>` y el contenido del contenedor de página, y actualiza la URL
 * con `history.pushState`. `popstate` (atrás/adelante) hace lo mismo sin
 * volver a empujar el historial.
 *
 * Contenedor de página estable (Fase 32): si el proyecto usa
 * `src/layout.tsx`, solo se reemplaza el contenido de `data-nexa-slot`
 * — el header/nav/footer del layout nunca se desmonta ni se vuelve a
 * montar en una navegación. Sin layout (o si por algún motivo la página
 * de destino no trae el mismo slot), se sigue reemplazando `<body>`
 * entero, igual que siempre — retrocompatible con cualquier proyecto sin
 * layout compartido.
 */
export function initRouter(options: InitRouterOptions = {}): () => void {
    const root = options.root ?? document;
    const fetchPage = options.fetchPage ?? defaultFetcher;
    const cache = options.cache ?? new Map<string, string>();
    const onNavigate = options.onNavigate;

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

        const reactivationRoot = applyPage(root, html);
        // Todo el documento, no solo `reactivationRoot`: un nav dentro
        // del layout (fuera del slot) también necesita esto, y es
        // justamente el caso que el slot-only swap deja sin cubrir.
        updateActiveLinks(root, path);
        onNavigate?.(reactivationRoot);
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

/** Devuelve el contenedor real que cambió — lo que `reactivate()` debe re-escanear. */
function applyPage(root: Document, html: string): ParentNode {
    const parsed = new DOMParser().parseFromString(html, "text/html");
    root.title = parsed.title;
    // Bug real (#26): antes de esto, una navegación SPA nunca tocaba
    // `<head>` — el `<link rel="stylesheet">` de `@nexa/ui` de la página
    // nueva (un archivo distinto en `nexa dev`, donde el CSS es solo el
    // de esa página; el mismo archivo compartido en `nexa build`, así
    // que ahí no se notaba) nunca se agregaba, y `canonical`/`meta`
    // (SEO real) se quedaban con los valores de la página anterior —
    // esto último sí importa en producción, con o sin layout.
    syncHead(root, parsed);

    const currentSlot = root.querySelector(SLOT_SELECTOR);
    const incomingSlot = parsed.querySelector(SLOT_SELECTOR);

    if (currentSlot && incomingSlot) {
        currentSlot.innerHTML = incomingSlot.innerHTML;
        // El manifiesto de activación (Fase 5) vive fuera del slot, al
        // final de <body> — sobrevive el swap de arriba tal cual, pero
        // sigue siendo el de la página ANTERIOR. Sin actualizarlo a
        // mano, `reactivate()` reactivaría el contenido nuevo con el
        // manifiesto viejo (ids que ya no corresponden a nada, o que
        // faltan).
        syncManifestScript(root, parsed);
        return currentSlot;
    }

    // Sin layout (o la página de destino no declara el mismo slot) —
    // mismo comportamiento de siempre: reemplazar <body> entero. Esto
    // ya trae consigo el manifiesto correcto, como parte normal del
    // HTML de la página de destino.
    root.body.innerHTML = parsed.body.innerHTML;
    return root;
}

const STYLESHEET_SELECTOR = 'link[rel="stylesheet"]';
const HEAD_SYNC_SELECTOR = [
    'link[rel="canonical"]',
    'link[rel="alternate"][hreflang]',
    "meta[name]",
    "meta[property]",
    'script[type="application/ld+json"]',
].join(", ");

/**
 * Sincroniza `<head>` con la página de destino — el mismo patrón que
 * `syncManifestScript` (comparar viejo vs. nuevo, actualizar solo lo que
 * cambió), aplicado a los nodos de `<head>` que sí varían por página:
 * hojas de estilo, `canonical`, `hreflang`, `meta` con `name`/`property`,
 * y el JSON-LD de `schema`. No toca `<meta charset>` (no tiene
 * `name`/`property`, así que ningún selector de acá lo mira) ni los
 * `<link>` de favicon/apple-touch-icon (tampoco matchean). `viewport`/
 * `generator` sí entran por `meta[name]` — en la práctica es un
 * reemplazo por un clon idéntico, porque toda página real de Nexa los
 * declara igual, así que no hace falta un caso especial para excluirlos.
 */
function syncHead(root: Document, parsed: Document): void {
    syncStylesheets(root, parsed);
    syncHeadElementsByKey(root, parsed);
}

function syncStylesheets(root: Document, parsed: Document): void {
    const currentLinks = Array.from(root.head.querySelectorAll<HTMLLinkElement>(STYLESHEET_SELECTOR));
    const incomingLinks = Array.from(parsed.head.querySelectorAll<HTMLLinkElement>(STYLESHEET_SELECTOR));

    const currentHrefs = new Set(currentLinks.map((link) => link.href));
    const incomingHrefs = new Set(incomingLinks.map((link) => link.href));

    for (const link of incomingLinks) {
        if (!currentHrefs.has(link.href)) {
            root.head.appendChild(link.cloneNode(true));
        }
    }
    for (const link of currentLinks) {
        if (!incomingHrefs.has(link.href)) {
            link.remove();
        }
    }
}

/**
 * Identidad estable de un nodo de `<head>` sincronizable — lo que hace
 * que `<meta name="description">` de la página vieja y la nueva sean
 * "el mismo elemento" a reemplazar, sin importar que el `content`
 * cambie. `null` para cualquier cosa fuera de las cinco formas que
 * `HEAD_SYNC_SELECTOR` ya filtró (nunca debería pasar, pero evita un
 * `undefined` silencioso si el selector se extiende a futuro).
 */
function headElementKey(el: Element): string | null {
    const tag = el.tagName.toLowerCase();
    if (tag === "link") {
        const rel = el.getAttribute("rel");
        if (rel === "canonical") return "link:canonical";
        if (rel === "alternate") return `link:alternate:${el.getAttribute("hreflang") ?? ""}`;
        return null;
    }
    if (tag === "meta") {
        const name = el.getAttribute("name");
        if (name) return `meta:name:${name}`;
        const property = el.getAttribute("property");
        if (property) return `meta:property:${property}`;
        return null;
    }
    if (tag === "script" && el.getAttribute("type") === "application/ld+json") {
        return "script:ld-json";
    }
    return null;
}

function syncHeadElementsByKey(root: Document, parsed: Document): void {
    const current = new Map<string, Element>();
    for (const el of Array.from(root.head.querySelectorAll(HEAD_SYNC_SELECTOR))) {
        const key = headElementKey(el);
        if (key) current.set(key, el);
    }

    const seen = new Set<string>();
    for (const el of Array.from(parsed.head.querySelectorAll(HEAD_SYNC_SELECTOR))) {
        const key = headElementKey(el);
        if (!key) continue;
        seen.add(key);

        const existing = current.get(key);
        if (existing) {
            existing.replaceWith(el.cloneNode(true));
        } else {
            root.head.appendChild(el.cloneNode(true));
        }
    }

    for (const [key, el] of current) {
        if (!seen.has(key)) el.remove();
    }
}

function syncManifestScript(root: Document, parsed: Document): void {
    const current = root.querySelector(MANIFEST_SELECTOR);
    const incoming = parsed.querySelector(MANIFEST_SELECTOR);

    if (incoming) {
        if (current) {
            current.textContent = incoming.textContent;
        } else {
            root.body.appendChild(incoming.cloneNode(true));
        }
    } else {
        current?.remove();
    }
}
