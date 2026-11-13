/**
 * Tipos de JSX para el Modo App. Un proyecto que quiera autocompletado
 * en `src/app/**` agrega `"jsx": "preserve"` a su tsconfig y referencia
 * este archivo (`/// <reference types="@nexa/app/src/jsx" />`).
 *
 * Los atributos son deliberadamente laxos (`any`): una prop puede ser
 * un valor, una `Signal` o una función `() => valor` (reactiva).
 */
declare namespace JSX {
    type Element = Node;
    interface IntrinsicElements {
        [tag: string]: Record<string, any>;
    }
    interface ElementChildrenAttribute {
        children: unknown;
    }
}
