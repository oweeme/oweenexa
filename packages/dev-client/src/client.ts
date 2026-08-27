import { createPoller, type PollerOptions } from "./poller";

export interface InitDevClientOptions extends PollerOptions {
    /** Cada cuánto sondear `/__nexa_dev__/version`, en milisegundos. */
    intervalMs?: number;
}

/**
 * Arranca el sondeo de auto-reload de `nexa dev`. Pensado para ejecutarse
 * desde un `<script>` en `<head>` (lo inyecta `nexa-cli`, ver
 * `commands::dev`): así el intervalo sigue vivo incluso después de que
 * el propio código reemplace el `<body>`, que es donde vive todo el
 * contenido de la página.
 */
export function initDevClient(options: InitDevClientOptions = {}): () => void {
    const intervalMs = options.intervalMs ?? 400;
    const poller = createPoller(options);

    void poller.tick();
    const id = setInterval(() => void poller.tick(), intervalMs);

    return () => clearInterval(id);
}
