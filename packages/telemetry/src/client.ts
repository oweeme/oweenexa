import { observeErrors } from "./errors";
import { defaultReportSender, type Report, type ReportSender } from "./report";
import { observeVitals } from "./vitals";

export interface InitTelemetryOptions {
    /**
     * A dónde enviar los reportes. `nexa-cli` solo inyecta este módulo
     * en absoluto si `nexa.toml` declara `[telemetry] endpoint = "..."`
     * (Fase 14) — sin endpoint, no hay ninguna razón para que este
     * código llegue al navegador siquiera.
     */
    endpoint: string;
    /**
     * En vez de (o además de) enviar al endpoint, entrega cada reporte
     * aquí — pensado para pruebas, o para que el proyecto decida su
     * propio destino sin depender de la red.
     */
    onReport?: (report: Report) => void;
    sendReport?: ReportSender;
    document?: Document;
    window?: Window;
}

/**
 * Arranca la telemetría: Core Web Vitals + errores sin capturar, cada
 * uno enviado a `endpoint` en cuanto se mide (no se acumulan en un lote
 * — un reporte perdido si la pestaña se cierra abruptamente es
 * aceptable; un lote entero perdido no lo es).
 */
export function initTelemetry(options: InitTelemetryOptions): () => void {
    const sendReport = options.sendReport ?? defaultReportSender;
    const doc = options.document ?? document;
    const path = doc.defaultView?.location.pathname ?? "/";

    const emit = (report: Report) => {
        options.onReport?.(report);
        sendReport(options.endpoint, report);
    };

    const stopVitals = observeVitals((vital) => {
        emit({ kind: "vital", name: vital.name, value: vital.value, path, timestamp: Date.now() });
    });

    const stopErrors = observeErrors(
        (error) => {
            emit({ kind: "error", name: error.name, detail: error.message, path, timestamp: Date.now() });
        },
        { window: options.window },
    );

    return () => {
        stopVitals();
        stopErrors();
    };
}
