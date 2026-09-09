import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { connectSSE, connectSocket } from "../src";

class FakeEventSource {
    onopen: ((event: Event) => void) | null = null;
    onmessage: ((event: { data: string }) => void) | null = null;
    onerror: ((event: unknown) => void) | null = null;
    closed = false;

    constructor(public readonly url: string) {}

    emit(data: string): void {
        this.onmessage?.({ data });
    }

    fail(event: unknown): void {
        this.onerror?.(event);
    }

    close(): void {
        this.closed = true;
    }
}

class FakeWebSocket {
    listeners: Record<string, Array<(event: unknown) => void>> = {};
    sent: unknown[] = [];
    closed = false;

    constructor(public readonly url: string) {}

    addEventListener(type: string, handler: (event: unknown) => void): void {
        (this.listeners[type] ??= []).push(handler);
    }

    emit(type: string, event: unknown = {}): void {
        for (const handler of this.listeners[type] ?? []) handler(event);
    }

    send(data: unknown): void {
        this.sent.push(data);
    }

    close(): void {
        this.closed = true;
    }
}

describe("connectSSE", () => {
    it("parsea cada mensaje como JSON en data", () => {
        let source!: FakeEventSource;
        const conn = connectSSE<{ price: number }>({
            url: "/events",
            eventSourceImpl: function (url: string) {
                source = new FakeEventSource(url);
                return source as unknown as EventSource;
            } as unknown as typeof EventSource,
        });

        expect(conn.data.value).toBeUndefined();
        source.emit(JSON.stringify({ price: 42 }));
        expect(conn.data.value).toEqual({ price: 42 });
    });

    it("cae al texto crudo si el mensaje no es JSON válido", () => {
        let source!: FakeEventSource;
        const conn = connectSSE<string>({
            url: "/events",
            eventSourceImpl: function (url: string) {
                source = new FakeEventSource(url);
                return source as unknown as EventSource;
            } as unknown as typeof EventSource,
        });

        source.emit("no soy json");
        expect(conn.data.value).toBe("no soy json");
    });

    it("expone el error real cuando la conexión falla", () => {
        let source!: FakeEventSource;
        const conn = connectSSE({
            url: "/events",
            eventSourceImpl: function (url: string) {
                source = new FakeEventSource(url);
                return source as unknown as EventSource;
            } as unknown as typeof EventSource,
        });

        const fakeErrorEvent = { type: "error" };
        source.fail(fakeErrorEvent);
        expect(conn.error.value).toBe(fakeErrorEvent);
    });

    it("close() cierra el EventSource real", () => {
        let source!: FakeEventSource;
        const conn = connectSSE({
            url: "/events",
            eventSourceImpl: function (url: string) {
                source = new FakeEventSource(url);
                return source as unknown as EventSource;
            } as unknown as typeof EventSource,
        });

        conn.close();
        expect(source.closed).toBe(true);
    });
});

describe("connectSocket", () => {
    function fakeSocketImpl(capture: (socket: FakeWebSocket) => void): typeof WebSocket {
        return function (url: string) {
            const socket = new FakeWebSocket(url);
            capture(socket);
            return socket as unknown as WebSocket;
        } as unknown as typeof WebSocket;
    }

    it("empieza en connecting y pasa a open cuando el socket real abre", () => {
        let socket!: FakeWebSocket;
        const conn = connectSocket({ url: "wss://x", webSocketImpl: fakeSocketImpl((s) => (socket = s)) });

        expect(conn.status.value).toBe("connecting");
        socket.emit("open");
        expect(conn.status.value).toBe("open");
    });

    it("pasa a closed cuando el socket real cierra", () => {
        let socket!: FakeWebSocket;
        const conn = connectSocket({ url: "wss://x", webSocketImpl: fakeSocketImpl((s) => (socket = s)) });

        socket.emit("close");
        expect(conn.status.value).toBe("closed");
    });

    it("parsea cada mensaje entrante como JSON en data", () => {
        let socket!: FakeWebSocket;
        const conn = connectSocket<{ id: number }>({
            url: "wss://x",
            webSocketImpl: fakeSocketImpl((s) => (socket = s)),
        });

        socket.emit("message", { data: JSON.stringify({ id: 7 }) });
        expect(conn.data.value).toEqual({ id: 7 });
    });

    it("send() serializa objetos como JSON, pero manda un string tal cual", () => {
        let socket!: FakeWebSocket;
        const conn = connectSocket({ url: "wss://x", webSocketImpl: fakeSocketImpl((s) => (socket = s)) });

        conn.send({ type: "ping" });
        conn.send("raw-string");

        expect(socket.sent).toEqual([JSON.stringify({ type: "ping" }), "raw-string"]);
    });

    it("close() cierra el WebSocket real", () => {
        let socket!: FakeWebSocket;
        const conn = connectSocket({ url: "wss://x", webSocketImpl: fakeSocketImpl((s) => (socket = s)) });

        conn.close();
        expect(socket.closed).toBe(true);
    });
});

// Fase 35 — reconexión con backoff, opt-in vía `reconnect`.
describe("connectSocket — reconexión (Fase 35)", () => {
    function fakeSocketImpl(capture: (socket: FakeWebSocket) => void): typeof WebSocket {
        return function (url: string) {
            const socket = new FakeWebSocket(url);
            capture(socket);
            return socket as unknown as WebSocket;
        } as unknown as typeof WebSocket;
    }

    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("sin `reconnect`, un corte de conexión sigue pasando a closed definitivo (retrocompatible)", () => {
        let socket!: FakeWebSocket;
        const conn = connectSocket({ url: "wss://x", webSocketImpl: fakeSocketImpl((s) => (socket = s)) });

        socket.emit("close");
        expect(conn.status.value).toBe("closed");
        vi.advanceTimersByTime(60_000);
        expect(conn.status.value).toBe("closed");
    });

    it("con `reconnect`, un corte pasa a reconnecting y abre un socket nuevo tras el backoff", () => {
        const sockets: FakeWebSocket[] = [];
        const conn = connectSocket({
            url: "wss://x",
            webSocketImpl: fakeSocketImpl((s) => sockets.push(s)),
            reconnect: { maxAttempts: 3, baseDelayMs: 100, maxDelayMs: 1000 },
        });

        expect(sockets).toHaveLength(1);
        sockets[0].emit("close");
        expect(conn.status.value).toBe("reconnecting");
        expect(sockets).toHaveLength(1); // todavía no pasó el delay

        vi.advanceTimersByTime(1000); // >= el máximo posible con jitter para intento 0
        expect(sockets).toHaveLength(2);
        sockets[1].emit("open");
        expect(conn.status.value).toBe("open");
    });

    it("agotados los maxAttempts, status pasa a closed definitivo y no abre más sockets", () => {
        const sockets: FakeWebSocket[] = [];
        const conn = connectSocket({
            url: "wss://x",
            webSocketImpl: fakeSocketImpl((s) => sockets.push(s)),
            reconnect: { maxAttempts: 2, baseDelayMs: 10, maxDelayMs: 100 },
        });

        sockets[0].emit("close"); // intento 1
        vi.advanceTimersByTime(100);
        expect(sockets).toHaveLength(2);

        sockets[1].emit("close"); // intento 2
        vi.advanceTimersByTime(100);
        expect(sockets).toHaveLength(3);

        sockets[2].emit("close"); // ya se agotaron los maxAttempts (2)
        expect(conn.status.value).toBe("closed");
        vi.advanceTimersByTime(10_000);
        expect(sockets).toHaveLength(3); // ningún socket nuevo después de agotarse
    });

    it("un reintento exitoso reinicia el contador de intentos", () => {
        const sockets: FakeWebSocket[] = [];
        const conn = connectSocket({
            url: "wss://x",
            webSocketImpl: fakeSocketImpl((s) => sockets.push(s)),
            reconnect: { maxAttempts: 1, baseDelayMs: 10, maxDelayMs: 100 },
        });

        sockets[0].emit("close");
        vi.advanceTimersByTime(100);
        expect(sockets).toHaveLength(2);
        sockets[1].emit("open"); // reconectó bien — el contador vuelve a 0

        sockets[1].emit("close"); // un segundo corte, ya con el contador reseteado
        expect(conn.status.value).toBe("reconnecting"); // no "closed": todavía tenía intentos
        vi.advanceTimersByTime(100);
        expect(sockets).toHaveLength(3);
    });

    it("cerrar a mano con close() no dispara ningún intento de reconexión", () => {
        const sockets: FakeWebSocket[] = [];
        const conn = connectSocket({
            url: "wss://x",
            webSocketImpl: fakeSocketImpl((s) => sockets.push(s)),
            reconnect: true,
        });

        conn.close();
        sockets[0].emit("close"); // un WebSocket real dispara esto solo, después de .close()
        expect(conn.status.value).toBe("closed");
        vi.advanceTimersByTime(60_000);
        expect(sockets).toHaveLength(1);
    });

    it("`reconnect: true` usa los valores por defecto (maxAttempts: 10)", () => {
        const sockets: FakeWebSocket[] = [];
        connectSocket({ url: "wss://x", webSocketImpl: fakeSocketImpl((s) => sockets.push(s)), reconnect: true });

        for (let i = 0; i < 10; i++) {
            sockets[sockets.length - 1].emit("close");
            vi.advanceTimersByTime(15_000); // maxDelayMs por defecto
        }
        // 1 inicial + 10 reintentos permitidos por el default = 11.
        expect(sockets).toHaveLength(11);
    });
});

describe("connectSSE — reconexión (Fase 35)", () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it("sin `reconnect`, un error no cierra el EventSource nativo (deja que reintente solo)", () => {
        let source!: FakeEventSource;
        const conn = connectSSE({
            url: "/events",
            eventSourceImpl: function (url: string) {
                source = new FakeEventSource(url);
                return source as unknown as EventSource;
            } as unknown as typeof EventSource,
        });

        source.fail({ type: "error" });
        expect(source.closed).toBe(false);
        expect(conn.status.value).toBe("connecting");
    });

    it("con `reconnect`, un error cierra el EventSource nativo y abre uno nuevo tras el backoff", () => {
        const sources: FakeEventSource[] = [];
        const conn = connectSSE({
            url: "/events",
            eventSourceImpl: function (url: string) {
                const s = new FakeEventSource(url);
                sources.push(s);
                return s as unknown as EventSource;
            } as unknown as typeof EventSource,
            reconnect: { maxAttempts: 3, baseDelayMs: 100, maxDelayMs: 1000 },
        });

        sources[0].fail({ type: "error" });
        expect(sources[0].closed).toBe(true);
        expect(conn.status.value).toBe("reconnecting");

        vi.advanceTimersByTime(1000);
        expect(sources).toHaveLength(2);
        sources[1].onopen?.(undefined as unknown as Event);
        expect(conn.status.value).toBe("open");
    });

    it("agotados los maxAttempts, status pasa a closed definitivo", () => {
        const sources: FakeEventSource[] = [];
        const conn = connectSSE({
            url: "/events",
            eventSourceImpl: function (url: string) {
                const s = new FakeEventSource(url);
                sources.push(s);
                return s as unknown as EventSource;
            } as unknown as typeof EventSource,
            reconnect: { maxAttempts: 1, baseDelayMs: 10, maxDelayMs: 100 },
        });

        sources[0].fail({ type: "error" });
        vi.advanceTimersByTime(100);
        expect(sources).toHaveLength(2);

        sources[1].fail({ type: "error" });
        expect(conn.status.value).toBe("closed");
        vi.advanceTimersByTime(10_000);
        expect(sources).toHaveLength(2);
    });
});
