export interface ErrorReport {
    name: "error" | "unhandledrejection";
    message: string;
}

export type ErrorCallback = (report: ErrorReport) => void;

export interface ObserveErrorsOptions {
    window?: Window;
}

/**
 * `window.onerror` para excepciones sin capturar, y `unhandledrejection`
 * para promesas rechazadas sin `.catch()` — las dos fuentes de "algo
 * rompió y nadie se enteró" que un navegador expone de forma nativa.
 * Solo se reporta el mensaje (más archivo/línea si el navegador los da):
 * nunca cookies, ni el estado de la app, ni nada que el desarrollador no
 * haya puesto ya en el mensaje de error él mismo.
 */
export function observeErrors(onError: ErrorCallback, options: ObserveErrorsOptions = {}): () => void {
    const win = options.window ?? (typeof window !== "undefined" ? window : undefined);
    if (!win) {
        return () => {};
    }

    const handleError = (event: ErrorEvent) => {
        const location = event.filename ? ` (${event.filename}:${event.lineno}:${event.colno})` : "";
        onError({ name: "error", message: `${event.message}${location}` });
    };

    const handleRejection = (event: PromiseRejectionEvent) => {
        const reason = event.reason instanceof Error ? event.reason.message : String(event.reason);
        onError({ name: "unhandledrejection", message: reason });
    };

    win.addEventListener("error", handleError);
    win.addEventListener("unhandledrejection", handleRejection);

    return () => {
        win.removeEventListener("error", handleError);
        win.removeEventListener("unhandledrejection", handleRejection);
    };
}
