/**
 * Espejo, en TypeScript, de lo que `nexa-activation` serializa como
 * `dist/nexa-manifest.json` (crates/nexa-activation/src/manifest.rs y
 * strategy.rs). Si esa forma cambia allá, debe cambiar aquí.
 */
export type Strategy = "interaction" | "visible" | "idle" | "load" | "manual";

export interface ActivationEntry {
    event: string;
    handler: string;
    module: string;
    strategy: Strategy;
}

/** Clave = id de nodo (`data-nexa="<id>"`), como texto. */
export type ActivationManifest = Record<string, ActivationEntry>;
