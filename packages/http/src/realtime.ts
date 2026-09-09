import { state, type Signal } from "@nexa/reactivity";

/**
 * Envoltorio reactivo delgado sobre `EventSource`/`WebSocket` nativos —
 * ningún protocolo propio. El punto es el mismo patrón que
 * `query()`/`mutation()`: un `Signal` que una isla (o cualquier handler)
 * puede leer con `effect()`/`bindText()`, en vez de que cada proyecto
 * reinvente su propio wrapper de eventos del navegador.
 *
 * Reconexión (Fase 35): opt-in vía `reconnect` — sin declararlo, el
 * comportamiento es exactamente el de antes (retrocompatible): un
 * `WebSocket` que se corta queda `"closed"` para siempre, y un
 * `EventSource` reintenta solo con el mecanismo nativo del navegador
 * (sin backoff configurable ni límite de intentos).
 */

export interface ReconnectOptions {
    /** Default 10. */
    maxAttempts?: number;
    /** Delay del primer reintento, en ms. Default 500. */
    baseDelayMs?: number;
    /** Tope del delay entre reintentos, en ms. Default 15000. */
    maxDelayMs?: number;
}

interface NormalizedReconnect {
    maxAttempts: number;
    baseDelayMs: number;
    maxDelayMs: number;
}

const DEFAULT_RECONNECT: NormalizedReconnect = { maxAttempts: 10, baseDelayMs: 500, maxDelayMs: 15000 };

function normalizeReconnect(input: boolean | ReconnectOptions | undefined): NormalizedReconnect | undefined {
    if (!input) return undefined;
    if (input === true) return DEFAULT_RECONNECT;
    return {
        maxAttempts: input.maxAttempts ?? DEFAULT_RECONNECT.maxAttempts,
        baseDelayMs: input.baseDelayMs ?? DEFAULT_RECONNECT.baseDelayMs,
        maxDelayMs: input.maxDelayMs ?? DEFAULT_RECONNECT.maxDelayMs,
    };
}

/**
 * Backoff exponencial con jitter (mitad fija + mitad aleatoria, para que
 * muchos clientes reconectando a la vez no lo hagan todos en el mismo
 * instante exacto). `attempt` empieza en 0 para el primer reintento.
 */
function backoffDelay(attempt: number, baseDelayMs: number, maxDelayMs: number): number {
    const exp = Math.min(maxDelayMs, baseDelayMs * 2 ** attempt);
    return exp / 2 + Math.random() * (exp / 2);
}

export type SSEStatus = "connecting" | "open" | "reconnecting" | "closed";

export interface SSEOptions {
    url: string;
    /** Inyectable para tests; por defecto, el `EventSource` global. */
    eventSourceImpl?: typeof EventSource;
    reconnect?: boolean | ReconnectOptions;
}

export interface SSEConnection<T> {
    /** El último mensaje recibido, parseado como JSON si es posible —
     * si no, el texto crudo del evento. `undefined` hasta el primero. */
    data: Signal<T | undefined>;
    status: Signal<SSEStatus>;
    error: Signal<unknown>;
    close: () => void;
}

export function connectSSE<T = unknown>(options: SSEOptions): SSEConnection<T> {
    const EventSourceImpl = options.eventSourceImpl ?? EventSource;
    const data = state<T | undefined>(undefined);
    const status = state<SSEStatus>("connecting");
    const error = state<unknown>(undefined);
    const reconnectConfig = normalizeReconnect(options.reconnect);

    let source: EventSource;
    let closedByUser = false;
    let attempt = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;

    function open(): void {
        source = new EventSourceImpl(options.url);
        source.onopen = () => {
            attempt = 0;
            status.value = "open";
        };
        source.onmessage = (event) => {
            data.value = parseMessage<T>(event.data);
        };
        source.onerror = (event) => {
            error.value = event;
            if (closedByUser || !reconnectConfig) {
                // Sin `reconnect` configurado, se deja que el
                // `EventSource` nativo reintente solo — comportamiento
                // de siempre, sin backoff ni límite de intentos.
                return;
            }
            // Se cierra el nativo para que no reintente por su cuenta
            // (sin backoff configurable) y se maneja la reconexión a mano.
            source.close();
            if (attempt < reconnectConfig.maxAttempts) {
                status.value = "reconnecting";
                const delay = backoffDelay(attempt, reconnectConfig.baseDelayMs, reconnectConfig.maxDelayMs);
                attempt++;
                timer = setTimeout(open, delay);
            } else {
                status.value = "closed";
            }
        };
    }

    open();

    return {
        data,
        status,
        error,
        close: () => {
            closedByUser = true;
            if (timer) clearTimeout(timer);
            source.close();
            status.value = "closed";
        },
    };
}

export type SocketStatus = "connecting" | "open" | "reconnecting" | "closed";

export interface SocketOptions {
    url: string;
    /** Inyectable para tests; por defecto, el `WebSocket` global. */
    webSocketImpl?: typeof WebSocket;
    reconnect?: boolean | ReconnectOptions;
}

export interface SocketConnection<T> {
    data: Signal<T | undefined>;
    status: Signal<SocketStatus>;
    error: Signal<unknown>;
    /** Un string se manda tal cual; cualquier otro valor se serializa
     * como JSON primero. Opera siempre sobre la conexión vigente —
     * si hubo una reconexión, es la nueva, no la que se cortó. */
    send: (message: unknown) => void;
    close: () => void;
}

export function connectSocket<T = unknown>(options: SocketOptions): SocketConnection<T> {
    const WebSocketImpl = options.webSocketImpl ?? WebSocket;
    const data = state<T | undefined>(undefined);
    const status = state<SocketStatus>("connecting");
    const error = state<unknown>(undefined);
    const reconnectConfig = normalizeReconnect(options.reconnect);

    let socket: WebSocket;
    let closedByUser = false;
    let attempt = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;

    function open(): void {
        socket = new WebSocketImpl(options.url);
        socket.addEventListener("open", () => {
            attempt = 0;
            status.value = "open";
        });
        socket.addEventListener("close", () => {
            if (closedByUser) {
                status.value = "closed";
                return;
            }
            if (reconnectConfig && attempt < reconnectConfig.maxAttempts) {
                status.value = "reconnecting";
                const delay = backoffDelay(attempt, reconnectConfig.baseDelayMs, reconnectConfig.maxDelayMs);
                attempt++;
                timer = setTimeout(open, delay);
            } else {
                status.value = "closed";
            }
        });
        socket.addEventListener("error", (event) => {
            error.value = event;
        });
        socket.addEventListener("message", (event) => {
            data.value = parseMessage<T>((event as MessageEvent).data);
        });
    }

    open();

    return {
        data,
        status,
        error,
        send: (message) => socket.send(typeof message === "string" ? message : JSON.stringify(message)),
        close: () => {
            closedByUser = true;
            if (timer) clearTimeout(timer);
            socket.close();
        },
    };
}

function parseMessage<T>(raw: unknown): T {
    if (typeof raw !== "string") {
        return raw as T;
    }
    try {
        return JSON.parse(raw) as T;
    } catch {
        return raw as unknown as T;
    }
}
