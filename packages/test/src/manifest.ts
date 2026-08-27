/** Espejo de `dist/<ruta>/nexa-manifest.json` — lo que `nexa-activation`
 * serializa (`ActivationManifest`, Fase 5). */
export interface ManifestEntry {
    event: string;
    handler: string;
    module: string;
    strategy: "interaction" | "visible" | "idle" | "load" | "manual";
}

export type Manifest = Record<string, ManifestEntry>;

/**
 * La entrada de `nodeId`, o lanza con un mensaje que lista los ids que
 * de verdad hay — más útil en un fallo de test que un `undefined` mudo.
 */
export function getEntry(manifest: Manifest, nodeId: string | number): ManifestEntry {
    const entry = manifest[String(nodeId)];
    if (!entry) {
        const known = Object.keys(manifest);
        throw new Error(
            `[nexa/test] no hay ninguna entrada para el nodo ${nodeId} en el manifiesto. ` +
                `Ids presentes: ${known.length > 0 ? known.join(", ") : "(ninguno — el manifiesto está vacío)"}.`,
        );
    }
    return entry;
}

/**
 * Comprueba que la entrada de `nodeId` coincide con lo esperado — solo
 * en los campos que se pasen; los demás no se comprueban. Lanza con un
 * mensaje que muestra ambos lados en desacuerdo, no solo "false".
 */
export function expectEntry(manifest: Manifest, nodeId: string | number, expected: Partial<ManifestEntry>): void {
    const actual = getEntry(manifest, nodeId);

    for (const [key, value] of Object.entries(expected) as Array<[keyof ManifestEntry, unknown]>) {
        if (actual[key] !== value) {
            throw new Error(
                `[nexa/test] nodo ${nodeId}: se esperaba ${key} = ${JSON.stringify(value)}, ` +
                    `pero era ${JSON.stringify(actual[key])}.`,
            );
        }
    }
}
