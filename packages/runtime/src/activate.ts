import type { ActivationManifest } from "./manifest";
import { idle, interaction, load, manual, visible, type StrategyRunner } from "./strategies";

/**
 * Cómo cargar el módulo de un chunk. Por defecto, un `import()` dinámico
 * de verdad; se puede inyectar otra cosa (por ejemplo en tests) para no
 * depender de tener un bundler real sirviendo esos archivos.
 */
export type ModuleLoader = (path: string) => Promise<{ default: (el: Element) => void }>;

const defaultLoader: ModuleLoader = (path) => import(/* @vite-ignore */ path);

const runners: Record<string, StrategyRunner> = { interaction, visible, idle, load, manual };

export interface InitActivationOptions {
    root?: ParentNode;
    loadModule?: ModuleLoader;
}

/**
 * Recorre el DOM buscando `[data-nexa]` y, para cada uno, programa su
 * activación según lo que diga el manifiesto — sin cargar ningún módulo
 * hasta que la estrategia correspondiente decide que toca. Un documento
 * sin ningún `data-nexa` (contenido 100% estático) no llama a
 * `loadModule` ni una sola vez.
 *
 * Devuelve una función para cancelar todas las activaciones pendientes.
 */
export function initActivation(
    manifest: ActivationManifest,
    options: InitActivationOptions = {},
): () => void {
    const root = options.root ?? document;
    const loadModule = options.loadModule ?? defaultLoader;

    const disposers: Array<() => void> = [];

    for (const [id, entry] of Object.entries(manifest)) {
        // `querySelectorAll`, no `querySelector`: un mismo id puede
        // aparecer en más de un elemento cuando sale de un `<For>` (Fase
        // 30) — el body se clasifica una sola vez, así que todas las
        // copias que produce el renderer comparten el mismo
        // `data-nexa="<id>"`. Cada copia se activa por separado, con su
        // propio `el` (y por lo tanto su propio `event.currentTarget`
        // cuando el handler necesita saber sobre qué elemento actuó).
        const elements = root.querySelectorAll(`[data-nexa="${id}"]`);
        if (elements.length === 0) continue;

        for (const el of Array.from(elements)) {
            const trigger = async () => {
                const mod = await loadModule(entry.module);
                mod.default(el);
            };

            const runner = runners[entry.strategy] ?? interaction;
            disposers.push(runner(el, entry.event, trigger));
        }
    }

    return () => {
        for (const dispose of disposers) dispose();
    };
}
