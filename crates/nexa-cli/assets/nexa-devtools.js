// packages/devtools/src/panel.ts
var STYLE = `
    all: initial;
    position: fixed;
    right: 12px;
    bottom: 12px;
    z-index: 2147483647;
    font-family: ui-monospace, Menlo, Consolas, monospace;
    font-size: 12px;
`;
function createPanelContainer(doc) {
  const el = doc.createElement("div");
  el.setAttribute("data-nexa-devtools", "1");
  el.style.cssText = STYLE;
  doc.body.appendChild(el);
  return el;
}
function render(container, diagnostics) {
  const warningCount = diagnostics.seoWarnings.length + diagnostics.pkgWarnings.length;
  const badgeColor = warningCount > 0 ? "#ff6b6b" : "#4caf82";
  const jsKb = (diagnostics.initialJsBytes / 1024).toFixed(1);
  container.innerHTML = `
        <details style="background:#1e1e1e;color:#f2f2f2;border-radius:8px;box-shadow:0 2px 12px rgba(0,0,0,.4);overflow:hidden">
            <summary style="list-style:none;cursor:pointer;padding:6px 10px;display:flex;align-items:center;gap:6px;user-select:none">
                <span style="width:8px;height:8px;border-radius:50%;background:${badgeColor};display:inline-block"></span>
                <span>Nexa \xB7 ${diagnostics.classification.total} nodos \xB7 ${jsKb}kb JS inicial${warningCount > 0 ? ` \xB7 ${warningCount} aviso(s)` : ""}</span>
            </summary>
            <div style="padding:10px;max-width:360px;max-height:50vh;overflow:auto;border-top:1px solid #333">
                ${section("Clasificaci\xF3n", classificationRows(diagnostics))}
                ${section("Activaci\xF3n", activationRows(diagnostics))}
                ${section("SEO", warningRows(diagnostics.seoWarnings))}
                ${section("Paquetes", warningRows(diagnostics.pkgWarnings))}
            </div>
        </details>
    `;
}
function section(title, body) {
  return `<div style="margin-bottom:8px"><div style="color:#888;margin-bottom:2px">${title}</div>${body}</div>`;
}
function classificationRows(d) {
  const c = d.classification;
  return `<div>static ${c.static} \xB7 dynamic ${c.dynamic} \xB7 interactive ${c.interactive} \xB7 async ${c.async}</div>`;
}
function activationRows(d) {
  if (d.activation.length === 0) return `<div>\u2014</div>`;
  return d.activation.map((a) => `<div>#${escapeHtml(a.id)} ${escapeHtml(a.event)} \u2192 ${escapeHtml(a.handler)} (${escapeHtml(a.strategy)})</div>`).join("");
}
function warningRows(warnings) {
  if (warnings.length === 0) return `<div>ninguno</div>`;
  return warnings.map((w) => `<div>[${escapeHtml(w.code)}] ${escapeHtml(w.message)}</div>`).join("");
}
function escapeHtml(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// packages/devtools/src/diagnostics.ts
var defaultDiagnosticsFetcher = (path) => fetch(`/__nexa_dev__/diagnostics?path=${encodeURIComponent(path)}`, { cache: "no-store" }).then(
  (res) => res.json()
);

// packages/devtools/src/poller.ts
function createPoller(options = {}) {
  const fetchDiagnostics = options.fetchDiagnostics ?? defaultDiagnosticsFetcher;
  const doc = options.document ?? document;
  const onDiagnostics = options.onDiagnostics ?? (() => {
  });
  let lastPath = null;
  async function tick(force = false) {
    const path = doc.defaultView?.location.pathname ?? "/";
    if (path === lastPath && !force) return;
    try {
      const diagnostics = await fetchDiagnostics(path);
      lastPath = path;
      onDiagnostics(diagnostics);
    } catch {
    }
  }
  return { tick };
}

// packages/devtools/src/client.ts
function initDevtools(options = {}) {
  const intervalMs = options.intervalMs ?? 1e3;
  const doc = options.document ?? document;
  let container = null;
  const poller = createPoller({
    ...options,
    onDiagnostics: (diagnostics) => {
      if (!container || !container.isConnected) {
        container = createPanelContainer(doc);
      }
      render(container, diagnostics);
    }
  });
  const tick = () => void poller.tick(!container || !container.isConnected);
  tick();
  const id = setInterval(tick, intervalMs);
  return () => clearInterval(id);
}
export {
  createPanelContainer,
  createPoller,
  defaultDiagnosticsFetcher,
  initDevtools,
  render
};
