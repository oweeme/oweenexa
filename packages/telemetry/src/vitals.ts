/**
 * Core Web Vitals sobre `PerformanceObserver` nativo — sin la librería
 * `web-vitals` (para no añadir una dependencia externa al bundle de un
 * solo archivo que usa `nexa-cli`, ver `packages/*` en general). Esto es
 * una simplificación honesta, no una reimplementación completa:
 *
 * - **LCP**: el candidato más grande hasta que la página deja de estar
 *   visible o hay una interacción — el algoritmo real (`web-vitals`
 *   ajusta por `visibilitychange` y descarta entradas post-interacción
 *   con más matices que esto).
 * - **CLS**: suma de `value` en entradas `layout-shift` sin
 *   `hadRecentInput` — no agrupa en "sesiones" de layout shift como hace
 *   el algoritmo oficial (`web-vitals` sí lo hace).
 * - **INP**: aproximado con `first-input` (la señal de FID, ahora
 *   retirada) en vez del algoritmo real de INP (percentil 98 de todas
 *   las interacciones de la página) — ese algoritmo es
 *   significativamente más complejo de implementar correctamente sin la
 *   librería oficial.
 *
 * Cada observer es opcional: si el navegador no soporta un `entryType`,
 * simplemente no se reporta esa métrica — nunca un valor inventado.
 */

export type VitalName = "LCP" | "CLS" | "INP";

export interface VitalReport {
    name: VitalName;
    value: number;
}

export type VitalCallback = (report: VitalReport) => void;

export interface ObserveVitalsOptions {
    /** Inyectable para pruebas — un DOM de pruebas no dispara estas
     * entradas de rendimiento de verdad. */
    PerformanceObserverCtor?: typeof PerformanceObserver;
}

/** Devuelve una función para desconectar todos los observers. */
export function observeVitals(onVital: VitalCallback, options: ObserveVitalsOptions = {}): () => void {
    const Ctor = options.PerformanceObserverCtor ?? (typeof PerformanceObserver !== "undefined" ? PerformanceObserver : undefined);
    if (!Ctor) {
        return () => {};
    }

    const observers: PerformanceObserver[] = [];
    const supported = Ctor.supportedEntryTypes ?? [];

    let lastLcp = 0;
    observeType(Ctor, supported, "largest-contentful-paint", observers, (entries) => {
        const last = entries[entries.length - 1] as PerformanceEntry & { renderTime?: number; loadTime?: number };
        if (!last) return;
        lastLcp = last.renderTime || last.loadTime || last.startTime;
        onVital({ name: "LCP", value: lastLcp });
    });

    let cls = 0;
    observeType(Ctor, supported, "layout-shift", observers, (entries) => {
        for (const entry of entries as Array<PerformanceEntry & { value: number; hadRecentInput: boolean }>) {
            if (!entry.hadRecentInput) {
                cls += entry.value;
            }
        }
        onVital({ name: "CLS", value: cls });
    });

    observeType(Ctor, supported, "first-input", observers, (entries) => {
        const first = entries[0] as PerformanceEntry & { processingStart: number; startTime: number };
        if (!first) return;
        onVital({ name: "INP", value: first.processingStart - first.startTime });
    });

    return () => {
        for (const observer of observers) observer.disconnect();
    };
}

function observeType(
    Ctor: typeof PerformanceObserver,
    supported: readonly string[],
    type: string,
    observers: PerformanceObserver[],
    onEntries: (entries: PerformanceEntry[]) => void,
): void {
    if (supported.length > 0 && !supported.includes(type)) return;

    try {
        const observer = new Ctor((list) => onEntries(list.getEntries()));
        observer.observe({ type, buffered: true });
        observers.push(observer);
    } catch {
        // Un `entryType` que el navegador dice soportar pero rechaza al
        // observar (pasa en algunos navegadores más viejos) no debe
        // tumbar el resto de la telemetría.
    }
}
