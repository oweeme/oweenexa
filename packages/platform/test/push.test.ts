import { describe, expect, it, vi } from "vitest";

import { onActionPerformed, onReceived, register } from "../src/push";

function fakeCapacitorPushPlugin() {
    const listeners = new Map<string, (arg: unknown) => void>();
    const removeMocks = new Map<string, ReturnType<typeof vi.fn>>();
    return {
        register: vi.fn().mockResolvedValue(undefined),
        addListener: vi.fn().mockImplementation((event: string, listener: (arg: unknown) => void) => {
            listeners.set(event, listener);
            const remove = vi.fn().mockResolvedValue(undefined);
            removeMocks.set(event, remove);
            return Promise.resolve({ remove });
        }),
        fire(event: string, payload: unknown) {
            listeners.get(event)?.(payload);
        },
        removeMockFor(event: string) {
            return removeMocks.get(event);
        },
    };
}

describe("register", () => {
    it("resolves with the real FCM/APNs token on Capacitor when registration succeeds", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorPushPlugin();

        const pending = register({}, { environment, capacitorPlugin: plugin });
        await new Promise((r) => setTimeout(r, 0));
        plugin.fire("registration", { value: "fcm-token-abc" });

        await expect(pending).resolves.toEqual({ platform: "capacitor", token: "fcm-token-abc" });
        expect(plugin.register).toHaveBeenCalled();
    });

    it("rejects with a clear error on Capacitor when registration fails", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorPushPlugin();

        const pending = register({}, { environment, capacitorPlugin: plugin });
        await new Promise((r) => setTimeout(r, 0));
        plugin.fire("registrationError", { error: "no permission" });

        await expect(pending).rejects.toThrow(/no permission/);
    });

    it("cleans up both listeners once one of them settles", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorPushPlugin();

        const pending = register({}, { environment, capacitorPlugin: plugin });
        await new Promise((r) => setTimeout(r, 0));
        plugin.fire("registration", { value: "tok" });
        await pending;

        expect(plugin.removeMockFor("registration")).toHaveBeenCalled();
        expect(plugin.removeMockFor("registrationError")).toHaveBeenCalled();
    });

    it("throws a clear error when Capacitor is native but the plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(register({}, { environment })).rejects.toThrow(/@capacitor\/push-notifications/);
    });

    it("subscribes via the real PushManager on web", async () => {
        const subscribe = vi.fn().mockResolvedValue({ toJSON: () => ({ endpoint: "https://push.example/1", keys: { p256dh: "a", auth: "b" } }) });
        const registration = { pushManager: { subscribe } };

        const result = await register(
            { vapidPublicKey: "BEl62iUYgUivxIkv69yViEuiBIa40HI" },
            { environment: {}, navigatorObject: { serviceWorker: {} as ServiceWorkerContainer }, serviceWorkerRegistration: registration as unknown as ServiceWorkerRegistration },
        );

        expect(subscribe).toHaveBeenCalledWith(expect.objectContaining({ userVisibleOnly: true }));
        expect(result).toEqual({ platform: "web", subscription: { endpoint: "https://push.example/1", keys: { p256dh: "a", auth: "b" } } });
    });

    it("throws a clear error on web when no vapidPublicKey is given", async () => {
        await expect(register({}, { environment: {}, navigatorObject: { serviceWorker: {} as ServiceWorkerContainer } })).rejects.toThrow(
            /vapidPublicKey/,
        );
    });

    it("throws a clear error on web when there's no registered service worker", async () => {
        const nav = { serviceWorker: { getRegistration: vi.fn().mockResolvedValue(undefined) } };
        await expect(
            register({ vapidPublicKey: "abc" }, { environment: {}, navigatorObject: nav as unknown as Pick<Navigator, "serviceWorker"> }),
        ).rejects.toThrow(/Service Worker registrado/);
    });

    it("throws a clear error when Service Workers aren't available at all", async () => {
        await expect(register({ vapidPublicKey: "abc" }, { environment: {}, navigatorObject: {} })).rejects.toThrow(/Service Workers/);
    });
});

describe("onReceived", () => {
    it("subscribes to the real Capacitor pushNotificationReceived event", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorPushPlugin();
        const callback = vi.fn();

        onReceived(callback, { environment, capacitorPlugin: plugin });
        await new Promise((r) => setTimeout(r, 0));
        plugin.fire("pushNotificationReceived", { id: "1", data: {} });

        expect(callback).toHaveBeenCalledWith({ id: "1", data: {} });
    });

    it("removes the Capacitor listener when unsubscribed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorPushPlugin();

        const unsubscribe = onReceived(vi.fn(), { environment, capacitorPlugin: plugin });
        await new Promise((r) => setTimeout(r, 0));
        unsubscribe();

        expect(plugin.removeMockFor("pushNotificationReceived")).toHaveBeenCalled();
    });

    it("throws a clear, actionable error on web instead of silently doing nothing", () => {
        expect(() => onReceived(vi.fn(), { environment: {} })).toThrow(/Service Worker/);
    });
});

describe("onActionPerformed", () => {
    it("subscribes to the real Capacitor pushNotificationActionPerformed event", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const plugin = fakeCapacitorPushPlugin();
        const callback = vi.fn();

        onActionPerformed(callback, { environment, capacitorPlugin: plugin });
        await new Promise((r) => setTimeout(r, 0));
        plugin.fire("pushNotificationActionPerformed", { actionId: "tap" });

        expect(callback).toHaveBeenCalledWith({ actionId: "tap" });
    });

    it("throws a clear, actionable error on web instead of silently doing nothing", () => {
        expect(() => onActionPerformed(vi.fn(), { environment: {} })).toThrow(/notificationclick/);
    });
});
