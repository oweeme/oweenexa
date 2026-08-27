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
        const el = root.querySelector(`[data-nexa="${id}"]`);
        if (!el) continue;

        const trigger = async () => {
            const mod = await loadModule(entry.module);
            mod.default(el);
        };

        const runner = runners[entry.strategy] ?? interaction;
        disposers.push(runner(el, entry.event, trigger));
    }

    return () => {
        for (const dispose of disposers) dispose();
    };
}
