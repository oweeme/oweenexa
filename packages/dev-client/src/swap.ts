/** Cómo pedir el HTML completo de la página actual. */
export type PageFetcher = (path: string) => Promise<string>;

export const defaultPageFetcher: PageFetcher = (path) =>
    fetch(path, { cache: "no-store" }).then((res) => res.text());

/**
 * Reemplaza el `<body>` del documento por el de `html`, preservando el
 * scroll, y re-ejecuta los `<script>` del nuevo body.
 *
 * Un `<script>` insertado vía `innerHTML` nunca se ejecuta — es
 * comportamiento estándar del DOM, no un descuido — así que hay que
 * sacarlo del HTML antes de asignarlo, y luego recrearlo con
 * `createElement`/`appendChild` (eso sí se ejecuta). Es lo que hace que
 * el bootstrap de la página nueva (router, activación, `@nexa/forms`)
 * vuelva a correr con el manifiesto correcto de *esta* versión del
 * código, en vez de quedarse con el de la versión anterior.
 */
export function swapDocument(doc: Document, html: string): void {
    const parsed = new DOMParser().parseFromString(html, "text/html");
    const scripts = Array.from(parsed.body.querySelectorAll("script"));
    for (const script of scripts) {
        script.remove();
    }

    const scrollX = doc.defaultView?.scrollX ?? 0;
    const scrollY = doc.defaultView?.scrollY ?? 0;

    doc.title = parsed.title;
    doc.body.innerHTML = parsed.body.innerHTML;

    for (const script of scripts) {
        const fresh = doc.createElement("script");
        for (const attr of Array.from(script.attributes)) {
            fresh.setAttribute(attr.name, attr.value);
        }
        fresh.textContent = script.textContent;
        doc.body.appendChild(fresh);
    }

    doc.defaultView?.scrollTo(scrollX, scrollY);
}
