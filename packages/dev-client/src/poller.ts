import { defaultPageFetcher, swapDocument, type PageFetcher } from "./swap";
import { defaultVersionFetcher, type VersionFetcher } from "./version";

export interface PollerOptions {
    fetchVersion?: VersionFetcher;
    fetchPage?: PageFetcher;
    document?: Document;
}

export interface Poller {
    /**
     * Un sondeo: la primera llamada solo fija la versión de referencia
     * (no hay "cambio" contra el que compararla todavía). Las siguientes
     * comparan contra esa referencia; si cambió, recarga la página
     * actual y adopta la nueva versión como referencia.
     */
    tick: () => Promise<void>;
}

export function createPoller(options: PollerOptions = {}): Poller {
    const fetchVersion = options.fetchVersion ?? defaultVersionFetcher;
    const fetchPage = options.fetchPage ?? defaultPageFetcher;
    const doc = options.document ?? document;

    let baseline: string | null = null;

    async function tick(): Promise<void> {
        try {
            const version = await fetchVersion();

            if (baseline === null) {
                baseline = version;
                return;
            }
            if (version === baseline) {
                return;
            }

            baseline = version;
            const path = doc.defaultView?.location.pathname ?? "/";
            const html = await fetchPage(path);
            swapDocument(doc, html);
        } catch {
            // El servidor de dev puede estar recompilando o caído un
            // instante — se reintenta en el siguiente sondeo, sin ruido
            // en la consola por cada petición que falle.
        }
    }

    return { tick };
}
