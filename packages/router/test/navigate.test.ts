import { afterEach, describe, expect, it, vi } from "vitest";
import { initRouter, type PageFetcher } from "../src";

function pageHtml(title: string, bodyHtml: string): string {
    return `<!doctype html><html><head><title>${title}</title></head><body>${bodyHtml}</body></html>`;
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
