import { createPanelContainer, render } from "./panel";
import { createPoller, type PollerOptions } from "./poller";

export interface InitDevtoolsOptions extends PollerOptions {
    /** Cada cuánto comprobar si la ruta cambió, en milisegundos. */
    intervalMs?: number;
}

/**
 * Arranca el panel de diagnóstico de `nexa dev`. Igual que
 * `@nexa/dev-client`, pensado para ejecutarse desde un `<script>` en
 * `<head>` — así el panel sigue vivo aunque `<body>` se reemplace
 * (auto-reload) o se navegue por SPA.
 */
export function initDevtools(options: InitDevtoolsOptions = {}): () => void {
    const intervalMs = options.intervalMs ?? 1000;
    const doc = options.document ?? document;

    let container: HTMLElement | null = null;
    const poller = createPoller({
        ...options,
        onDiagnostics: (diagnostics) => {
            if (!container || !container.isConnected) {
                container = createPanelContainer(doc);
            }
            render(container, diagnostics);
        },
    });

    // Si el `<div>` del panel desapareció (un auto-reload de
    // `@nexa/dev-client` reemplazó `<body>`, así la ruta no haya
    // cambiado), `force: true` obliga a volver a pedir diagnósticos y
    // recrearlo — si no, el panel se quedaría vacío para siempre tras el
    // primer auto-reload en la misma página.
    const tick = () => void poller.tick(!container || !container.isConnected);

    tick();
    const id = setInterval(tick, intervalMs);

    return () => clearInterval(id);
}
