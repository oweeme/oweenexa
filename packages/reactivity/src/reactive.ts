/**
 * Núcleo de reactividad fine-grained de Nexa (Fase 4).
 *
 * Sin Virtual DOM: una señal notifica directamente a los efectos que la
 * leyeron; cada efecto toca solo lo que le corresponde (ver `dom.ts`). No
 * hay "re-render" de un árbol de componentes — ver
 * docs/FASES-DE-CONSTRUCCION.md (Fase 4) y el punto 16 ("Estado") de
 * docs/Arquitectura SEO Completo Framework.md.
 */

interface Effect {
    deps: Set<Signal<unknown>>;
    execute(): void;
}

let activeEffect: Effect | null = null;

const pending = new Set<Effect>();
let flushScheduled = false;

/** Agrupa varias escrituras síncronas en un único reflush por microtask. */
function schedule(effect: Effect): void {
    pending.add(effect);
    if (flushScheduled) return;
    flushScheduled = true;
    queueMicrotask(flush);
}

function flush(): void {
    flushScheduled = false;
    const effects = Array.from(pending);
    pending.clear();
    for (const effect of effects) {
        effect.execute();
    }
}

export class Signal<T> {
    #value: T;
    #subscribers = new Set<Effect>();

    constructor(initial: T) {
        this.#value = initial;
    }

    get value(): T {
        if (activeEffect) {
            this.#subscribers.add(activeEffect);
            activeEffect.deps.add(this as Signal<unknown>);
        }
        return this.#value;
    }

    set value(next: T) {
        if (Object.is(next, this.#value)) return;
        this.#value = next;
        for (const effect of this.#subscribers) {
            schedule(effect);
        }
    }

    /** Lee el valor sin suscribirse: no crea una dependencia. */
    peek(): T {
        return this.#value;
    }

    /** Usado por los efectos al limpiar sus dependencias antes de re-ejecutarse. */
    unsubscribe(effect: Effect): void {
        this.#subscribers.delete(effect);
    }
}

/** `state(0)` — la primitiva de estado de Nexa. */
export function state<T>(initial: T): Signal<T> {
    return new Signal(initial);
}

class EffectImpl implements Effect {
    deps = new Set<Signal<unknown>>();
    #fn: () => void;
    #disposed = false;

    constructor(fn: () => void) {
        this.#fn = fn;
        this.execute();
    }

    execute(): void {
        if (this.#disposed) return;
        this.#cleanup();

        const previous = activeEffect;
        activeEffect = this;
        try {
            this.#fn();
        } finally {
            activeEffect = previous;
        }
    }

    #cleanup(): void {
        for (const dep of this.deps) {
            dep.unsubscribe(this);
        }
        this.deps.clear();
    }

    dispose(): void {
        this.#disposed = true;
        this.#cleanup();
        pending.delete(this);
    }
}

function createEffect(fn: () => void): () => void {
    const instance = new EffectImpl(fn);
    return () => instance.dispose();
}

/**
 * `effect(() => ...)` — vuelve a ejecutar la función cuando cambia
 * cualquier señal leída dentro de ella (batching por microtask). Se
 * ejecuta una vez inmediatamente al crearse. Devuelve una función para
 * desactivarlo.
 *
 * `effect.client(...)` es, por ahora, un alias exacto: todavía no existe
 * un renderizado en servidor que ejecute efectos (eso llega con la Data
 * Layer, Fase 7), así que no hay nada real que distinguir. Se deja
 * declarado porque la especificación original marca esta frontera
 * server/client como parte del contrato, y así ningún código que ya use
 * `effect.client` tendrá que cambiar cuando esa distinción exista.
 */
export const effect: {
    (fn: () => void): () => void;
    client(fn: () => void): () => void;
} = Object.assign(createEffect, { client: createEffect });

/** Vista de solo lectura de una `Signal` — lo que devuelve `computed()`,
 * para que nada fuera de este módulo pueda escribirle `.value` a mano
 * (un valor derivado siempre se recalcula solo, nunca se asigna). */
export interface ReadonlySignal<T> {
    readonly value: T;
    peek(): T;
}

/**
 * `computed(() => a.value + b.value)` — una señal de solo lectura,
 * derivada de otras. Se recalcula automáticamente cuando cambia
 * cualquier señal que lea adentro (mismo tracking que `effect`), y solo
 * notifica a quien la lea si el resultado realmente cambió (`Signal`
 * ya descarta una escritura idéntica vía `Object.is`) — encadenar
 * `computed(() => otroComputed.value * 2)` funciona igual que con
 * `state()`, porque por dentro sigue siendo una `Signal` de verdad.
 *
 * Encadenar computeds sí tiene un costo real a tener en cuenta: cada
 * salto (`state` -> `computed A` -> `computed B`) es un `effect` que se
 * reprograma vía microtask, así que una cadena de N pasos necesita N
 * vueltas de microtask para terminar de propagarse — no es un defecto
 * de `computed`, es el mismo comportamiento que tendría encadenar los
 * `effect()` a mano.
 */
export function computed<T>(fn: () => T): ReadonlySignal<T> {
    const signal = new Signal<T>(undefined as T);
    effect(() => {
        signal.value = fn();
    });
    return signal;
}

/**
 * `watch(() => count.value, (next, prev) => ...)` — a diferencia de
 * `effect`, NO corre `callback` al crearse: solo cuando `source()`
 * cambia de verdad después de eso (mismo criterio `Object.is` que
 * `Signal.value`). Útil para reaccionar a un cambio (navegar, hacer un
 * `fetch`, etc.) sin que ese código se dispare una vez de más al
 * montar. Devuelve una función para dejar de observar.
 */
export function watch<T>(source: () => T, callback: (value: T, previous: T | undefined) => void): () => void {
    let previous: T | undefined;
    let first = true;

    return effect(() => {
        const value = source();
        if (first) {
            first = false;
            previous = value;
            return;
        }
        if (!Object.is(value, previous)) {
            const old = previous;
            previous = value;
            callback(value, old);
        }
    });
}
