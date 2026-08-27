/**
 * Cuándo se dispara la carga real de un chunk, por estrategia. Ninguna
 * de estas funciones carga JS por sí misma — solo deciden *cuándo* llamar
 * a `trigger()`, que es quien realmente hace `import()` (ver `activate.ts`).
 */

export type Trigger = () => Promise<void>;

/** Cada estrategia devuelve una función para cancelarse (dejar de escuchar). */
export type StrategyRunner = (el: Element, event: string, trigger: Trigger) => () => void;

/**
 * Por defecto: no carga nada hasta la primera interacción real. El evento
 * que la disparó no se pierde — se reproduce sobre el elemento una vez
 * que el chunk ya está activado, para que esa primera interacción del
 * usuario también surta efecto.
 */
export const interaction: StrategyRunner = (el, event, trigger) => {
    const listener = (nativeEvent: Event) => {
        el.removeEventListener(event, listener);
        void trigger().then(() => {
            el.dispatchEvent(cloneEvent(nativeEvent));
        });
    };

    el.addEventListener(event, listener);
    return () => el.removeEventListener(event, listener);
};

/** Se activa cuando el elemento entra en el viewport. */
export const visible: StrategyRunner = (el, _event, trigger) => {
    const observer = new IntersectionObserver((entries) => {
        for (const entry of entries) {
            if (entry.isIntersecting) {
                observer.disconnect();
                void trigger();
            }
        }
    });
    observer.observe(el);
    return () => observer.disconnect();
};

/** Se activa cuando el navegador está inactivo. */
export const idle: StrategyRunner = (_el, _event, trigger) => {
    const requestIdle: typeof requestIdleCallback =
        typeof requestIdleCallback === "function"
            ? requestIdleCallback
            : (fn) => setTimeout(() => fn(makeIdleDeadline()), 1) as unknown as number;
    const cancelIdle: typeof cancelIdleCallback =
        typeof cancelIdleCallback === "function" ? cancelIdleCallback : (id) => clearTimeout(id);

    const handle = requestIdle(() => void trigger());
    return () => cancelIdle(handle);
};

/** Se activa de inmediato: útil para lo que se sabe que hará falta ya. */
export const load: StrategyRunner = (_el, _event, trigger) => {
    void trigger();
    return () => {};
};

const manualTriggers = new WeakMap<Element, Trigger>();

/** No se activa sola: queda registrada hasta que algo llame a `activateManually`. */
export const manual: StrategyRunner = (el, _event, trigger) => {
    manualTriggers.set(el, trigger);
    return () => manualTriggers.delete(el);
};

/** Dispara a mano la activación de un elemento con estrategia `manual`. */
export function activateManually(el: Element): Promise<void> {
    const trigger = manualTriggers.get(el);
    return trigger ? trigger() : Promise.resolve();
}

function cloneEvent(event: Event): Event {
    const EventCtor = event.constructor as typeof Event;
    try {
        return new EventCtor(event.type, event as unknown as EventInit);
    } catch {
        return new Event(event.type, { bubbles: event.bubbles, cancelable: event.cancelable });
    }
}

function makeIdleDeadline(): IdleDeadline {
    return {
        didTimeout: false,
        timeRemaining: () => 0,
    };
}
