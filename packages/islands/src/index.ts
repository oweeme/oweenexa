import { idle, interaction, load, manual, visible, type StrategyRunner } from "@nexa/runtime";

/**
 * El contrato que debe cumplir el módulo al que resuelve un
 * `data-nexa-island="<specifier>"`: un export por defecto que recibe el
 * elemento de montaje y sus props ya resueltas a JSON — la misma
 * convención de export por defecto que ya usan los chunks de activación
 * de eventos (`activate(el)`, Fase 5). Nexa nunca abre ni valida este
 * módulo: es un contrato documental, no algo que el compilador comprueba.
 * El valor de retorno, si es una función, se guarda como limpieza
 * (equivalente a un `unmount`).
 */
export type MountFn = (el: Element, props: Record<string, unknown>) => void | (() => void);

/**
 * Cómo cargar el módulo de una isla. Por defecto, un `import()` dinámico
 * real; se puede inyectar otra cosa (tests) para no depender de un
 * bundler real sirviendo esos archivos — igual que `ModuleLoader` en
 * `@nexa/runtime`.
 */
export type IslandModuleLoader = (specifier: string) => Promise<{ default: MountFn }>;

const defaultLoader: IslandModuleLoader = (specifier) => import(/* @vite-ignore */ specifier);

// Mismas estrategias que la activación de eventos (Fase 5) — una isla
// `visible`/`idle`/`load`/`manual` se comporta exactamente igual que un
// chunk de evento con esa estrategia. Solo cambia el *default*: una isla
// no tiene "el" evento obvio de disparo, así que sin
// `data-nexa-strategy` explícito se comporta como `visible`, no como
// `interaction`.
const runners: Record<string, StrategyRunner> = { interaction, visible, idle, load, manual };

export interface InitIslandsOptions {
    root?: ParentNode;
    loadModule?: IslandModuleLoader;
}

/**
 * Recorre el DOM buscando `[data-nexa-island]` y, para cada uno, programa
 * su montaje según la estrategia declarada — sin cargar ningún módulo
 * hasta que le toca. Un documento sin ninguna isla nunca llama a
 * `loadModule`.
 *
 * Devuelve una función para cancelar todos los montajes pendientes.
 */
export function initIslands(options: InitIslandsOptions = {}): () => void {
    const root = options.root ?? document;
    const loadModule = options.loadModule ?? defaultLoader;

    const disposers: Array<() => void> = [];

    for (const el of Array.from(root.querySelectorAll("[data-nexa-island]"))) {
        const specifier = el.getAttribute("data-nexa-island");
        if (!specifier) continue;

        const props = parseProps(el.getAttribute("data-nexa-props"));
        const strategy = el.getAttribute("data-nexa-strategy") ?? "visible";
        // `interaction` necesita un evento DOM real sobre el que
        // escuchar; una isla no tiene "el" evento obvio de un
        // `onClick`, así que se usa uno genérico.
        const triggerEvent = strategy === "interaction" ? "pointerdown" : "";

        const trigger = async () => {
            const mod = await loadModule(specifier);
            const cleanup = mod.default(el, props);
            if (typeof cleanup === "function") disposers.push(cleanup);
        };

        const runner = runners[strategy] ?? visible;
        disposers.push(runner(el, triggerEvent, trigger));
    }

    return () => {
        for (const dispose of disposers) dispose();
    };
}

function parseProps(raw: string | null): Record<string, unknown> {
    if (!raw) return {};
    try {
        const parsed: unknown = JSON.parse(raw);
        return parsed && typeof parsed === "object" ? (parsed as Record<string, unknown>) : {};
    } catch {
        return {};
    }
}
