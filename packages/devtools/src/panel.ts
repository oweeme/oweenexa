import type { Diagnostics } from "./diagnostics";

const STYLE = `
    all: initial;
    position: fixed;
    right: 12px;
    bottom: 12px;
    z-index: 2147483647;
    font-family: ui-monospace, Menlo, Consolas, monospace;
    font-size: 12px;
`;

/**
 * Crea el contenedor fijo del panel y lo añade a `doc.body` — se llama
 * una sola vez. El contenido se rellena aparte, con `render`, cada vez
 * que hay diagnósticos nuevos.
 */
export function createPanelContainer(doc: Document): HTMLElement {
    const el = doc.createElement("div");
    el.setAttribute("data-nexa-devtools", "1");
    el.style.cssText = STYLE;
    doc.body.appendChild(el);
    return el;
}

/** Insignia colapsada: cuenta rápida de nodos + avisos, con un color que
 * delata si hay algo que mirar (rojo si hay avisos, verde si no). */
export function render(container: HTMLElement, diagnostics: Diagnostics): void {
    const warningCount = diagnostics.seoWarnings.length + diagnostics.pkgWarnings.length;
    const badgeColor = warningCount > 0 ? "#ff6b6b" : "#4caf82";
    const jsKb = (diagnostics.initialJsBytes / 1024).toFixed(1);

    container.innerHTML = `
        <details style="background:#1e1e1e;color:#f2f2f2;border-radius:8px;box-shadow:0 2px 12px rgba(0,0,0,.4);overflow:hidden">
            <summary style="list-style:none;cursor:pointer;padding:6px 10px;display:flex;align-items:center;gap:6px;user-select:none">
                <span style="width:8px;height:8px;border-radius:50%;background:${badgeColor};display:inline-block"></span>
                <span>Nexa · ${diagnostics.classification.total} nodos · ${jsKb}kb JS inicial${warningCount > 0 ? ` · ${warningCount} aviso(s)` : ""}</span>
            </summary>
            <div style="padding:10px;max-width:360px;max-height:50vh;overflow:auto;border-top:1px solid #333">
                ${section("Clasificación", classificationRows(diagnostics))}
                ${section("Activación", activationRows(diagnostics))}
                ${section("SEO", warningRows(diagnostics.seoWarnings))}
                ${section("Paquetes", warningRows(diagnostics.pkgWarnings))}
            </div>
        </details>
    `;
}

function section(title: string, body: string): string {
    return `<div style="margin-bottom:8px"><div style="color:#888;margin-bottom:2px">${title}</div>${body}</div>`;
}

function classificationRows(d: Diagnostics): string {
    const c = d.classification;
    return `<div>static ${c.static} · dynamic ${c.dynamic} · interactive ${c.interactive} · async ${c.async}</div>`;
}

function activationRows(d: Diagnostics): string {
    if (d.activation.length === 0) return `<div>—</div>`;
    return d.activation
        .map((a) => `<div>#${escapeHtml(a.id)} ${escapeHtml(a.event)} → ${escapeHtml(a.handler)} (${escapeHtml(a.strategy)})</div>`)
        .join("");
}

function warningRows(warnings: Array<{ code: string; message: string }>): string {
    if (warnings.length === 0) return `<div>ninguno</div>`;
    return warnings.map((w) => `<div>[${escapeHtml(w.code)}] ${escapeHtml(w.message)}</div>`).join("");
}

function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
