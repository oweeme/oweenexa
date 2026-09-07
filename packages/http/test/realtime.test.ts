import { describe, expect, it } from "vitest";
import { connectSSE, connectSocket } from "../src";

class FakeEventSource {
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
