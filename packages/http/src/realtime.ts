import { state, type Signal } from "@nexa/reactivity";

/**
 * Envoltorio reactivo delgado sobre `EventSource`/`WebSocket` nativos —
 * ningún protocolo propio, ninguna reconexión automática todavía (ver
 * limitaciones). El punto es el mismo patrón que `query()`/`mutation()`:
 * un `Signal` que una isla (o cualquier handler) puede leer con
 * `effect()`/`bindText()`, en vez de que cada proyecto reinvente su
 * propio wrapper de eventos del navegador.
 */

export interface SSEOptions {
    url: string;
    /** Inyectable para tests; por defecto, el `EventSource` global. */
    eventSourceImpl?: typeof EventSource;
}

export interface SSEConnection<T> {
    /** El último mensaje recibido, parseado como JSON si es posible —
     * si no, el texto crudo del evento. `undefined` hasta el primero. */
    data: Signal<T | undefined>;
    error: Signal<unknown>;
    close: () => void;
}

export function connectSSE<T = unknown>(options: SSEOptions): SSEConnection<T> {
    const EventSourceImpl = options.eventSourceImpl ?? EventSource;
    const data = state<T | undefined>(undefined);
    const error = state<unknown>(undefined);

    const source = new EventSourceImpl(options.url);
    source.onmessage = (event) => {
        data.value = parseMessage<T>(event.data);
    };
    source.onerror = (event) => {
        error.value = event;
    };

    return { data, error, close: () => source.close() };
}

export type SocketStatus = "connecting" | "open" | "closed";

export interface SocketOptions {
    url: string;
    /** Inyectable para tests; por defecto, el `WebSocket` global. */
    webSocketImpl?: typeof WebSocket;
}

export interface SocketConnection<T> {
    data: Signal<T | undefined>;
    status: Signal<SocketStatus>;
    error: Signal<unknown>;
    /** Un string se manda tal cual; cualquier otro valor se serializa
     * como JSON primero. */
    send: (message: unknown) => void;
    close: () => void;
}

export function connectSocket<T = unknown>(options: SocketOptions): SocketConnection<T> {
    const WebSocketImpl = options.webSocketImpl ?? WebSocket;
    const data = state<T | undefined>(undefined);
    const status = state<SocketStatus>("connecting");
    const error = state<unknown>(undefined);

    const socket = new WebSocketImpl(options.url);
    socket.addEventListener("open", () => {
        status.value = "open";
    });
    socket.addEventListener("close", () => {
        status.value = "closed";
    });
    socket.addEventListener("error", (event) => {
        error.value = event;
    });
    socket.addEventListener("message", (event) => {
        data.value = parseMessage<T>((event as MessageEvent).data);
    });

    return {
        data,
        status,
        error,
        send: (message) => socket.send(typeof message === "string" ? message : JSON.stringify(message)),
        close: () => socket.close(),
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
