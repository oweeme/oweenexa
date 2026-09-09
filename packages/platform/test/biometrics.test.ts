import { describe, expect, it, vi } from "vitest";

import { authenticate, isAvailable } from "../src/biometrics";

function fakeStorage(): { get: ReturnType<typeof vi.fn>; set: ReturnType<typeof vi.fn>; remove: ReturnType<typeof vi.fn> } {
    const data = new Map<string, string>();
    return {
        get: vi.fn((key: string) => data.get(key) ?? null),
        set: vi.fn((key: string, value: string) => void data.set(key, value)),
        remove: vi.fn((key: string) => void data.delete(key)),
    };
}

describe("isAvailable", () => {
    it("calls the real Capacitor plugin when running natively", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const checkBiometry = vi.fn().mockResolvedValue({ isAvailable: true, biometryType: 2 });

        const result = await isAvailable({ environment, capacitorPlugin: { checkBiometry, authenticate: vi.fn() } });

        expect(checkBiometry).toHaveBeenCalled();
        expect(result).toEqual({ isAvailable: true, biometryType: "faceId" });
    });

    it("maps every real Capacitor BiometryType enum value", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const cases: Array<[number, string]> = [
            [0, "none"],
            [1, "touchId"],
            [2, "faceId"],
            [3, "fingerprint"],
            [4, "face"],
            [5, "iris"],
        ];
        for (const [value, expected] of cases) {
            const checkBiometry = vi.fn().mockResolvedValue({ isAvailable: true, biometryType: value });
            const result = await isAvailable({ environment, capacitorPlugin: { checkBiometry, authenticate: vi.fn() } });
            expect(result.biometryType).toBe(expected);
        }
    });

    it("throws a clear error when Capacitor is native but the plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(isAvailable({ environment })).rejects.toThrow(/@aparajita\/capacitor-biometric-auth/);
    });

    it("uses the real WebAuthn platform authenticator check on web", async () => {
        const isUserVerifyingPlatformAuthenticatorAvailable = vi.fn().mockResolvedValue(true);
        const result = await isAvailable({
            environment: {},
            windowObject: { PublicKeyCredential: { isUserVerifyingPlatformAuthenticatorAvailable } as unknown as typeof PublicKeyCredential },
        });

        expect(isUserVerifyingPlatformAuthenticatorAvailable).toHaveBeenCalled();
        expect(result).toEqual({ isAvailable: true, biometryType: "platform" });
    });

    it("returns not-available when WebAuthn isn't supported at all", async () => {
        const result = await isAvailable({ environment: {}, windowObject: {} });
        expect(result).toEqual({ isAvailable: false, biometryType: "none" });
    });
});

describe("authenticate", () => {
    it("calls the real Capacitor authenticate() with the given reason", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        const authenticateMock = vi.fn().mockResolvedValue(undefined);

        await authenticate({ reason: "Desbloquear la app" }, { environment, capacitorPlugin: { checkBiometry: vi.fn(), authenticate: authenticateMock } });

        expect(authenticateMock).toHaveBeenCalledWith({ reason: "Desbloquear la app" });
    });

    it("throws a clear error when Capacitor is native but the plugin is not installed", async () => {
        const environment = { Capacitor: { isNativePlatform: () => true } };
        await expect(authenticate({}, { environment })).rejects.toThrow(/@aparajita\/capacitor-biometric-auth/);
    });

    it("creates a real platform credential on first use, on web, and stores its id", async () => {
        const rawId = new Uint8Array([1, 2, 3, 4]).buffer;
        const create = vi.fn().mockResolvedValue({ rawId });
        const storage = fakeStorage();

        await authenticate({}, { environment: {}, navigatorObject: { credentials: { create, get: vi.fn() } as unknown as CredentialsContainer }, storage });

        expect(create).toHaveBeenCalledOnce();
        const publicKeyOptions = create.mock.calls[0][0].publicKey;
        expect(publicKeyOptions.authenticatorSelection).toEqual({ authenticatorAttachment: "platform", userVerification: "required" });
        expect(storage.set).toHaveBeenCalledWith("nexa-biometrics-credential-id", expect.any(String));
    });

    it("re-authenticates against the already-stored credential on subsequent calls, on web", async () => {
        const get = vi.fn().mockResolvedValue({ id: "assertion" });
        const storage = fakeStorage();
        storage.set("nexa-biometrics-credential-id", "AQIDBA=="); // base64 de [1,2,3,4]

        await authenticate({}, { environment: {}, navigatorObject: { credentials: { create: vi.fn(), get } as unknown as CredentialsContainer }, storage });

        expect(get).toHaveBeenCalledOnce();
        const publicKeyOptions = get.mock.calls[0][0].publicKey;
        expect(publicKeyOptions.allowCredentials[0].id).toEqual(new Uint8Array([1, 2, 3, 4]));
        expect(publicKeyOptions.userVerification).toBe("required");
    });

    it("throws a clear error when the platform credential creation is cancelled/fails", async () => {
        const create = vi.fn().mockResolvedValue(null);
        const storage = fakeStorage();

        await expect(
            authenticate({}, { environment: {}, navigatorObject: { credentials: { create, get: vi.fn() } as unknown as CredentialsContainer }, storage }),
        ).rejects.toThrow(/credencial biométrica/);
    });

    it("throws a clear error when re-authentication is cancelled/fails", async () => {
        const get = vi.fn().mockResolvedValue(null);
        const storage = fakeStorage();
        storage.set("nexa-biometrics-credential-id", "AQIDBA==");

        await expect(
            authenticate({}, { environment: {}, navigatorObject: { credentials: { create: vi.fn(), get } as unknown as CredentialsContainer }, storage }),
        ).rejects.toThrow(/autenticación biométrica falló/);
    });

    it("throws a clear error when WebAuthn is not available at all", async () => {
        await expect(authenticate({}, { environment: {}, navigatorObject: {} })).rejects.toThrow(/WebAuthn/);
    });
});
