import { effect } from "./reactive";

/**
 * Vincula un nodo de texto del DOM a una función reactiva: cuando cambia
 * alguna señal que `compute` lee, se actualiza *solo* `node.data` — nunca
 * se reconstruye el elemento contenedor ni se tocan sus hermanos.
 *
 * Esto es el germen de la Fase 5 (Progressive Activation): allí el
 * compilador generará estas llamadas automáticamente a partir de los
 * marcadores `<!--nexa:product.name-->` que ya produce `nexa-renderer`.
 * Por ahora se usa a mano — es exactamente lo que demuestra el criterio
 * de salida de la Fase 4.
 */
export function bindText(node: Text, compute: () => string | number): () => void {
    return effect(() => {
        node.data = String(compute());
    });
}
