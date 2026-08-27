import { defaultDiagnosticsFetcher, type Diagnostics, type DiagnosticsFetcher } from "./diagnostics";

export interface PollerOptions {
    fetchDiagnostics?: DiagnosticsFetcher;
    document?: Document;
    onDiagnostics?: (diagnostics: Diagnostics) => void;
}

export interface Poller {
    /**
     * Un sondeo: si `location.pathname` cambió desde el último (o es la
     * primera vez, o `force` es `true`), vuelve a pedir diagnósticos y
     * llama a `onDiagnostics`. Si no, no hace ninguna petición — el
     * panel no genera tráfico de más solo por estar abierto.
     *
     * `force` existe porque el panel puede necesitar redibujarse sin que
     * la ruta haya cambiado — ej. un auto-reload de `@nexa/dev-client`
     * en la misma página destruye el `<div>` del panel (vive en
     * `<body>`, que ese auto-reload reemplaza entero); quien conecta el
     * poller a la UI decide cuándo hace falta.
     */
    tick: (force?: boolean) => Promise<void>;
}

export function createPoller(options: PollerOptions = {}): Poller {
    const fetchDiagnostics = options.fetchDiagnostics ?? defaultDiagnosticsFetcher;
    const doc = options.document ?? document;
    const onDiagnostics = options.onDiagnostics ?? (() => {});

    let lastPath: string | null = null;

    async function tick(force = false): Promise<void> {
        const path = doc.defaultView?.location.pathname ?? "/";
        if (path === lastPath && !force) return;

        try {
            const diagnostics = await fetchDiagnostics(path);
            lastPath = path;
            onDiagnostics(diagnostics);
        } catch {
            // La página puede estar compilando (error de sintaxis) o el
            // servidor un instante caído — se reintenta en el próximo
            // sondeo, sin romper el panel.
        }
    }

    return { tick };
}
