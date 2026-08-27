/** Espejo, en TypeScript, de lo que sirve `/__nexa_dev__/diagnostics`
 * (`crates/nexa-cli/src/devtools.rs::diagnostics_json`). Si esa forma
 * cambia allá, debe cambiar aquí. */
export interface Diagnostics {
    pattern: string;
    classification: {
        static: number;
        dynamic: number;
        interactive: number;
        async: number;
        total: number;
    };
    initialJsBytes: number;
    activation: Array<{
        id: string;
        event: string;
        handler: string;
        module: string;
        strategy: string;
    }>;
    seoWarnings: Array<{ code: string; message: string }>;
    pkgWarnings: Array<{ code: string; message: string }>;
}

export type DiagnosticsFetcher = (path: string) => Promise<Diagnostics>;

export const defaultDiagnosticsFetcher: DiagnosticsFetcher = (path) =>
    fetch(`/__nexa_dev__/diagnostics?path=${encodeURIComponent(path)}`, { cache: "no-store" }).then((res) =>
        res.json(),
    );
