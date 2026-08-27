import { describe, expect, it, vi } from "vitest";

import { notify } from "../src/notifications";

describe("notify", () => {
    it("schedules through the Capacitor plugin when running natively", async () => {
        const schedule = vi.fn().mockResolvedValue(undefined);
        const environment = { Capacitor: { isNativePlatform: () => true } };

        await notify({ title: "Hola", body: "Mensaje" }, { environment, capacitorPlugin: { schedule } });

        expect(schedule).toHaveBeenCalledTimes(1);
        const arg = schedule.mock.calls[0]?.[0];
        expect(arg.notifications[0].title).toBe("Hola");
        expect(arg.notifications[0].body).toBe("Mensaje");
    });

    it("throws a clear error when Capacitor is native but the plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(notify({ title: "x" }, { environment })).rejects.toThrow(/local-notifications/);
    });

    it("uses the Notification constructor on web when permission is already granted", async () => {
        const requestPermission = vi.fn();
        class FakeNotification {
            static permission = "granted";
            static requestPermission = requestPermission;
            constructor(
                public title: string,
                public options?: NotificationOptions,
            ) {}
        }

        await notify(
            { title: "Hola", body: "Mensaje" },
            { environment: {}, notificationCtor: FakeNotification as unknown as typeof Notification },
        );

        expect(requestPermission).not.toHaveBeenCalled();
    });

    it("requests permission on web when not yet granted, and throws if denied", async () => {
        const requestPermission = vi.fn().mockResolvedValue("denied");
        class FakeNotification {
            static permission = "default";
            static requestPermission = requestPermission;
            constructor(public title: string) {}
        }

        await expect(
            notify(
                { title: "Hola" },
                { environment: {}, notificationCtor: FakeNotification as unknown as typeof Notification },
            ),
        ).rejects.toThrow(/denegado/);

        expect(requestPermission).toHaveBeenCalledTimes(1);
    });
});
