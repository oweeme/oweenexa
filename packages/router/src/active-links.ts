/**
 * Recalcula `aria-current="page"` en cada `<a href>` de `root` para que
 * coincida con `currentPath`, tras una navegación SPA (Fase 32/33).
 *
 * Por qué hace falta del lado del cliente: `aria-current` ya sale
 * resuelto del servidor en cada página (Fase 33, mismo mecanismo que
 * `seo`/`schema`) — pero desde que `initRouter` reemplaza solo
 * `data-nexa-slot` en vez de `<body>` entero (Fase 32), un nav que vive
 * en `src/layout.tsx` nunca se vuelve a renderizar del lado del
 * servidor en una navegación de cliente. Sin este recálculo, ese nav se
 * quedaría marcando para siempre la página con la que se cargó el sitio
 * la primera vez.
 *
 * Mismo criterio de coincidencia que el servidor
 * (`nexa-renderer::html::active_link_attr`): exacto por defecto,
 * `data-nexa-match="prefix"` (que el servidor SÍ deja en el HTML, a
 * diferencia de otros atributos de control, justo para que este código
 * lo pueda releer) para coincidencia por prefijo con límite de
 * segmento.
 */
export function updateActiveLinks(root: ParentNode, currentPath: string): void {
    for (const anchor of Array.from(root.querySelectorAll("a[href]"))) {
        const href = anchor.getAttribute("href");
        if (href === null) continue;

        const mode = anchor.getAttribute("data-nexa-match");
        const isActive = mode === "prefix" ? isPrefixMatch(href, currentPath) : href === currentPath;

        if (isActive) {
            anchor.setAttribute("aria-current", "page");
        } else {
            anchor.removeAttribute("aria-current");
        }
    }
}

function isPrefixMatch(href: string, currentPath: string): boolean {
    return currentPath === href || (currentPath.startsWith(href) && currentPath.slice(href.length).startsWith("/"));
}
