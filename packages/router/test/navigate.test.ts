import { afterEach, describe, expect, it, vi } from "vitest";
import { initRouter, type PageFetcher } from "../src";

function pageHtml(title: string, bodyHtml: string): string {
    return `<!doctype html><html><head><title>${title}</title></head><body>${bodyHtml}</body></html>`;
}

function pageHtmlWithHead(title: string, headExtra: string, bodyHtml: string): string {
    return `<!doctype html><html><head><title>${title}</title>${headExtra}</head><body>${bodyHtml}</body></html>`;
}

function link(href: string, attrs: Record<string, string> = {}): HTMLAnchorElement {
    const a = document.createElement("a");
    a.href = href;
    for (const [name, value] of Object.entries(attrs)) a.setAttribute(name, value);
    a.textContent = href;
    document.body.appendChild(a);
    return a;
}

function click(el: Element, init: Partial<MouseEventInit> = {}): void {
    el.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, button: 0, ...init }));
}

describe("initRouter — Fase 6, criterio de salida", () => {
    let dispose: (() => void) | undefined;

    afterEach(() => {
        dispose?.();
        dispose = undefined;
        document.body.innerHTML = "";
        document.head.innerHTML = "";
    });

    it("un clic en un enlace interno no recarga la página pero sí actualiza la URL", async () => {
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(pageHtml("Sobre", "<h1>Sobre nosotros</h1>"));
        const pushState = vi.spyOn(history, "pushState");

        const a = link("/about");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalledWith("/about"));

        expect(document.body.innerHTML).toContain("<h1>Sobre nosotros</h1>");
        expect(document.title).toBe("Sobre");
        // `anchor.href` ya viene resuelto a absoluto por el DOM — eso es lo
        // que se le pasa a `pushState` (sigue siendo same-origin y válido).
        expect(pushState).toHaveBeenCalledWith({}, "", expect.stringContaining("/about"));

        pushState.mockRestore();
    });

    it("usa la caché (prefetch) en vez de volver a pedir la página por red", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const cache = new Map<string, string>([["/about", pageHtml("Sobre", "<h1>Ya precargada</h1>")]]);

        const a = link("/about");
        dispose = initRouter({ fetchPage, cache });

        click(a);
        await Promise.resolve();
        await Promise.resolve();

        expect(fetchPage).not.toHaveBeenCalled();
        expect(document.body.innerHTML).toContain("Ya precargada");
    });

    it("ignora enlaces externos: no hace fetch ni preventDefault", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const a = link("https://external.example/otra-cosa");
        dispose = initRouter({ fetchPage });

        const event = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0 });
        a.dispatchEvent(event);
        await Promise.resolve();

        expect(event.defaultPrevented).toBe(false);
        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("ignora enlaces con target=_blank y con data-nexa-reload", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const blank = link("/otra", { target: "_blank" });
        const reload = link("/otra-mas", { "data-nexa-reload": "" });
        dispose = initRouter({ fetchPage });

        click(blank);
        click(reload);
        await Promise.resolve();

        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("ignora clics con teclas modificadoras (abrir en pestaña nueva, etc.)", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const a = link("/about");
        dispose = initRouter({ fetchPage });

        click(a, { ctrlKey: true });
        await Promise.resolve();

        expect(fetchPage).not.toHaveBeenCalled();
    });

    it("llama a onNavigate con el root después de reemplazar el body", async () => {
        // Bug real: sin esto, un <script> de la página de destino
        // insertado vía innerHTML nunca se ejecuta solo — el manifiesto
        // de esa página nunca se activa. `onNavigate` es lo que le
        // permite a quien arma el bootstrap reactivar el contenido nuevo.
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(pageHtml("Otra", "<button>x</button>"));
        const onNavigate = vi.fn();
        const a = link("/otra");
        dispose = initRouter({ fetchPage, onNavigate });

        click(a);
        await vi.waitFor(() => expect(onNavigate).toHaveBeenCalledTimes(1));

        expect(onNavigate).toHaveBeenCalledWith(document);
        // Para cuando se llama, el body ya tiene que estar reemplazado.
        expect(document.body.innerHTML).toContain("<button>x</button>");
    });

    it("llama a onNavigate también cuando sirve desde la caché de prefetch", async () => {
        const fetchPage = vi.fn<PageFetcher>();
        const cache = new Map<string, string>([["/otra", pageHtml("Otra", "<p>cache</p>")]]);
        const onNavigate = vi.fn();
        const a = link("/otra");
        dispose = initRouter({ fetchPage, cache, onNavigate });

        click(a);
        await Promise.resolve();
        await Promise.resolve();

        expect(onNavigate).toHaveBeenCalledTimes(1);
    });

    it("con un layout (data-nexa-slot), solo se reemplaza el slot — el resto del <body> queda intacto", async () => {
        // Fase 32: el header del layout, con lo que sea que tenga
        // montado (una isla, Fase 31), no debe tocarse en absoluto.
        document.body.innerHTML =
            '<header data-marker="original">Nav</header><div data-nexa-slot><p>Página 1</p></div>';
        const fetchPage = vi
            .fn<PageFetcher>()
            .mockResolvedValue(
                pageHtml(
                    "Otra",
                    '<header data-marker="original">Nav</header><div data-nexa-slot><p>Página 2</p></div>',
                ),
            );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        const header = document.querySelector("header");
        expect(header).not.toBeNull();
        // El header original nunca se destruyó — sigue siendo el MISMO
        // nodo del DOM, no una copia nueva con el mismo HTML.
        expect(header?.getAttribute("data-marker")).toBe("original");
        expect(document.querySelector("[data-nexa-slot]")?.innerHTML).toBe("<p>Página 2</p>");
    });

    it("onNavigate recibe el elemento del slot (no todo el documento) cuando el proyecto usa layout", async () => {
        document.body.innerHTML = '<header>Nav</header><div data-nexa-slot><p>1</p></div>';
        const fetchPage = vi
            .fn<PageFetcher>()
            .mockResolvedValue(pageHtml("Otra", '<header>Nav</header><div data-nexa-slot><button>x</button></div>'));
        const onNavigate = vi.fn();
        const a = link("/otra");
        dispose = initRouter({ fetchPage, onNavigate });

        click(a);
        await vi.waitFor(() => expect(onNavigate).toHaveBeenCalledTimes(1));

        const [receivedRoot] = onNavigate.mock.calls[0] as [ParentNode];
        expect(receivedRoot).toBe(document.querySelector("[data-nexa-slot]"));
        expect(receivedRoot).not.toBe(document);
    });

    it("el manifiesto de activación (fuera del slot) se actualiza al de la página de destino", async () => {
        document.body.innerHTML =
            '<div data-nexa-slot><p>1</p></div>' +
            '<script type="application/json" data-nexa-manifest>{"3":{"event":"click"}}</script>';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtml(
                "Otra",
                '<div data-nexa-slot><button>x</button></div>' +
                    '<script type="application/json" data-nexa-manifest>{"7":{"event":"click"}}</script>',
            ),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        const manifestEl = document.querySelector('script[data-nexa-manifest]');
        expect(manifestEl?.textContent).toBe('{"7":{"event":"click"}}');
    });

    it("un nav fuera del slot (en el layout) actualiza aria-current después de una navegación SPA", async () => {
        // Fase 33 + Fase 32 juntas: el layout nunca se vuelve a
        // renderizar del lado del servidor en una navegación de
        // cliente — sin este recálculo, este nav se quedaría marcando
        // para siempre la página con la que cargó el sitio.
        document.body.innerHTML =
            '<nav><a href="/" aria-current="page">Inicio</a><a href="/otra">Otra</a></nav>' +
            '<div data-nexa-slot><p>1</p></div>';
        const fetchPage = vi
            .fn<PageFetcher>()
            .mockResolvedValue(
                pageHtml(
                    "Otra",
                    '<nav><a href="/">Inicio</a><a href="/otra">Otra</a></nav><div data-nexa-slot><p>2</p></div>',
                ),
            );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.querySelector('nav a[href="/"]')?.hasAttribute("aria-current")).toBe(false);
        expect(document.querySelector('nav a[href="/otra"]')?.getAttribute("aria-current")).toBe("page");
    });

    it("sin data-nexa-slot en el destino, cae al reemplazo de <body> completo de siempre", async () => {
        // Retrocompatibilidad explícita: un proyecto sin layout (o cuya
        // página de destino no trae el mismo slot) sigue funcionando
        // exactamente como antes de la Fase 32.
        document.body.innerHTML = "<p>Página 1</p>";
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(pageHtml("Otra", "<h1>Página 2</h1>"));
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.body.innerHTML).toContain("<h1>Página 2</h1>");
    });

    it("Bug #26: agrega el <link rel=stylesheet> de la página de destino que la actual no tenía", async () => {
        document.head.innerHTML = '<link rel="stylesheet" href="/assets/nexa-ui.aaa.css">';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead("Otra", '<link rel="stylesheet" href="/assets/nexa-ui.bbb.css">', "<p>x</p>"),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        const hrefs = Array.from(document.head.querySelectorAll('link[rel="stylesheet"]')).map((l) =>
            l.getAttribute("href"),
        );
        expect(hrefs).toContain("/assets/nexa-ui.bbb.css");
    });

    it("Bug #26: quita el <link rel=stylesheet> de la página anterior que ya no está en la nueva", async () => {
        document.head.innerHTML = '<link rel="stylesheet" href="/assets/nexa-ui.aaa.css">';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead("Otra", '<link rel="stylesheet" href="/assets/nexa-ui.bbb.css">', "<p>x</p>"),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        const hrefs = Array.from(document.head.querySelectorAll('link[rel="stylesheet"]')).map((l) =>
            l.getAttribute("href"),
        );
        expect(hrefs).not.toContain("/assets/nexa-ui.aaa.css");
    });

    it("Bug #26: una hoja de estilo compartida (mismo href en ambas páginas) no se toca — sigue siendo el mismo nodo", async () => {
        document.head.innerHTML = '<link rel="stylesheet" href="/static/site.css">';
        const shared = document.head.querySelector("link");
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead("Otra", '<link rel="stylesheet" href="/static/site.css">', "<p>x</p>"),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.head.querySelector('link[rel="stylesheet"]')).toBe(shared);
    });

    it("Bug #26: actualiza canonical al de la página de destino", async () => {
        document.head.innerHTML = '<link rel="canonical" href="/es/pagina-vieja">';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead("Otra", '<link rel="canonical" href="/es/pagina-nueva">', "<p>x</p>"),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.querySelector('link[rel="canonical"]')?.getAttribute("href")).toBe("/es/pagina-nueva");
    });

    it("Bug #26: actualiza meta description y Open Graph al contenido de la página de destino", async () => {
        document.head.innerHTML =
            '<meta name="description" content="descripción vieja">' +
            '<meta property="og:title" content="Título viejo">';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead(
                "Otra",
                '<meta name="description" content="descripción nueva"><meta property="og:title" content="Título nuevo">',
                "<p>x</p>",
            ),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.querySelector('meta[name="description"]')?.getAttribute("content")).toBe("descripción nueva");
        expect(document.querySelector('meta[property="og:title"]')?.getAttribute("content")).toBe("Título nuevo");
    });

    it("Bug #26: agrega un meta que la página anterior no tenía, y quita uno que ya no está en la nueva", async () => {
        document.head.innerHTML = '<meta name="robots" content="noindex">';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead("Otra", '<meta property="og:image" content="/img/otra.jpg">', "<p>x</p>"),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.querySelector('meta[name="robots"]')).toBeNull();
        expect(document.querySelector('meta[property="og:image"]')?.getAttribute("content")).toBe("/img/otra.jpg");
    });

    it("Bug #26: sincroniza el JSON-LD (schema) al de la página de destino", async () => {
        document.head.innerHTML = '<script type="application/ld+json">{"@type":"WebPage"}</script>';
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(
            pageHtmlWithHead("Otra", '<script type="application/ld+json">{"@type":"Product"}</script>', "<p>x</p>"),
        );
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        expect(document.querySelector('script[type="application/ld+json"]')?.textContent).toBe('{"@type":"Product"}');
    });

    it("Bug #26: nunca toca <meta charset> — no tiene name/property, ningún selector de sincronización lo mira", async () => {
        document.head.innerHTML = '<meta charset="UTF-8"><meta name="viewport" content="width=device-width">';
        const charsetBefore = document.head.querySelector("meta[charset]");
        const fetchPage = vi
            .fn<PageFetcher>()
            .mockResolvedValue(pageHtmlWithHead("Otra", '<meta name="viewport" content="width=device-width">', "<p>x</p>"));
        const a = link("/otra");
        dispose = initRouter({ fetchPage });

        click(a);
        await vi.waitFor(() => expect(fetchPage).toHaveBeenCalled());

        // El charset ni se mira (no tiene name/property) — sigue siendo
        // el mismo nodo. El viewport sí pasa por meta[name], pero toda
        // página real de Nexa lo declara igual, así que el resultado es
        // un reemplazo por un clon idéntico.
        expect(document.head.querySelector("meta[charset]")).toBe(charsetBefore);
        expect(document.head.querySelector('meta[name="viewport"]')?.getAttribute("content")).toBe(
            "width=device-width",
        );
    });

    it("dispose() deja de interceptar clics", async () => {
        const fetchPage = vi.fn<PageFetcher>().mockResolvedValue(pageHtml("X", "<p>x</p>"));
        const a = link("/about");

        const stop = initRouter({ fetchPage });
        stop();

        click(a);
        await Promise.resolve();

        expect(fetchPage).not.toHaveBeenCalled();
    });
});
