export interface Report {
    /** `"vital"` (Core Web Vitals) o `"error"` (excepción sin capturar). */
    kind: "vital" | "error";
    /** Nombre de la métrica (`"LCP"`, `"CLS"`, `"INP"`) o del error
     * (`"error"`, `"unhandledrejection"`). */
    name: string;
    /** El valor numérico de la métrica — ausente para errores. */
    value?: number;
    /** Detalle adicional — el mensaje de un error, por ejemplo. Nunca
     * incluye cookies, identificadores de usuario, ni la query string
     * completa (privacidad: ver `docs/FASES-DE-CONSTRUCCION.md`). */
    detail?: string;
    /** `location.pathname` — deliberadamente no la URL completa (una
     * query string puede traer tokens u otros datos sensibles). */
    path: string;
    timestamp: number;
}

export type ReportSender = (endpoint: string, report: Report) => void;

/**
 * `navigator.sendBeacon` es la forma correcta de enviar telemetría al
 * salir de la página (no bloquea la navegación, el navegador garantiza
 * el envío) — con `fetch(..., { keepalive: true })` como respaldo donde
 * `sendBeacon` no esté disponible.
 */
export const defaultReportSender: ReportSender = (endpoint, report) => {
    const body = JSON.stringify(report);

    if (typeof navigator !== "undefined" && typeof navigator.sendBeacon === "function") {
        navigator.sendBeacon(endpoint, body);
        return;
    }

    if (typeof fetch === "function") {
        void fetch(endpoint, { method: "POST", body, keepalive: true }).catch(() => {
            // Sin conexión, endpoint caído, CORS... la telemetría nunca
            // debe romper la app ni generar ruido en la consola del
            // usuario final.
        });
    }
};
