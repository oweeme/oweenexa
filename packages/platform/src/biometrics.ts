import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";
import { createStorage, type KeyValueStorage } from "./storage";

export type BiometryType =
    | "none"
    | "touchId"
    | "faceId"
    | "fingerprint"
    | "face"
    | "iris"
    | "platform";

export interface BiometricsAvailability {
    isAvailable: boolean;
    biometryType: BiometryType;
}

export interface AuthenticateOptions {
    /**
     * Motivo mostrado en el diálogo nativo — solo Capacitor. En Web no
     * hay ningún parámetro de WebAuthn para inyectar texto: el diálogo
     * biométrico lo controla 100% el sistema operativo/navegador, por
     * seguridad (mismo motivo por el que no se puede personalizar la UI
     * de un pago con tarjeta).
     */
    reason?: string;
}

/** La forma real (recortada) de `checkBiometry`/`authenticate` en `@aparajita/capacitor-biometric-auth`. */
export interface CapacitorBiometricsPlugin {
    checkBiometry(): Promise<{ isAvailable: boolean; biometryType: number }>;
    authenticate(options?: { reason?: string }): Promise<void>;
}

export interface BiometricsDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorBiometricsPlugin;
    windowObject?: { PublicKeyCredential?: typeof PublicKeyCredential };
    navigatorObject?: Pick<Navigator, "credentials">;
    storage?: KeyValueStorage;
}

const STORAGE_KEY = "nexa-biometrics-credential-id";

/** `BiometryType` numérico real de `@aparajita/capacitor-biometric-auth` -> el nuestro. */
function mapCapacitorBiometryType(value: number): BiometryType {
    switch (value) {
        case 1:
            return "touchId";
        case 2:
            return "faceId";
        case 3:
            return "fingerprint";
        case 4:
            return "face";
        case 5:
            return "iris";
        default:
            return "none";
    }
}

/**
 * Capacitor nativo: el plugin real `@aparajita/capacitor-biometric-auth`
 * (`checkBiometry`). Web/Tauri: la disponibilidad real de un
 * autenticador de plataforma de WebAuthn
 * (`PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable()`)
 * — Face ID/Touch ID/Windows Hello/huella de Android, según lo que
 * tenga el dispositivo. **No** se replica la rama web del propio
 * `@aparajita/capacitor-biometric-auth`: leyendo su código fuente, esa
 * rama es una simulación deliberada para tests (un `confirm()` con
 * estado falso puesto a mano vía `setBiometryType()`), no una
 * capacidad real — usar WebAuthn real en su lugar es más fiel a la
 * disciplina del resto de `@nexa/platform` ("verificado contra algo
 * real, nunca inventado") que copiar esa simulación.
 */
export async function isAvailable(deps: BiometricsDeps = {}): Promise<BiometricsAvailability> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorBiometricsPlugin>(environment, "BiometricAuth");
        if (!plugin) {
            throw new Error("[nexa/platform] @aparajita/capacitor-biometric-auth no está instalado en esta app.");
        }
        const result = await plugin.checkBiometry();
        return { isAvailable: result.isAvailable, biometryType: mapCapacitorBiometryType(result.biometryType) };
    }

    const win = deps.windowObject ?? (typeof window !== "undefined" ? window : undefined);
    const ctor = win?.PublicKeyCredential;
    if (!ctor?.isUserVerifyingPlatformAuthenticatorAvailable) {
        return { isAvailable: false, biometryType: "none" };
    }

    const available = await ctor.isUserVerifyingPlatformAuthenticatorAvailable();
    return { isAvailable: available, biometryType: available ? "platform" : "none" };
}

/**
 * Pide autenticación biométrica real. Capacitor: `authenticate()` del
 * plugin real. Web: una ceremonia real de WebAuthn contra un
 * autenticador de plataforma — sin backend, sin servidor, un gesto
 * biométrico local nada más:
 *
 * - Primera vez: crea una credencial de plataforma real
 *   (`navigator.credentials.create`, `authenticatorAttachment:
 *   "platform"`, `userVerification: "required"`) — dispara el prompt
 *   biométrico real del sistema operativo. Su id se guarda con
 *   `platform.storage()` (Fase 38, reutilizado tal cual).
 * - Siguientes veces: reautentica contra esa misma credencial
 *   (`navigator.credentials.get`) — dispara el prompt de nuevo.
 *
 * El *challenge* es aleatorio y se descarta enseguida (no hay servidor
 * que lo verifique) — esto es deliberadamente un gate local del
 * dispositivo, no un login federado; para eso hace falta un backend
 * real verificando la firma, fuera de lo que puede hacer un módulo de
 * cliente.
 */
export async function authenticate(options: AuthenticateOptions = {}, deps: BiometricsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorBiometricsPlugin>(environment, "BiometricAuth");
        if (!plugin) {
            throw new Error("[nexa/platform] @aparajita/capacitor-biometric-auth no está instalado en esta app.");
        }
        await plugin.authenticate({ reason: options.reason });
        return;
    }

    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav?.credentials) {
        throw new Error("[nexa/platform] WebAuthn no está disponible en este entorno.");
    }

    const storage = deps.storage ?? createStorage();
    const storedId = storage.get(STORAGE_KEY);

    if (!storedId) {
        const credential = await nav.credentials.create({
            publicKey: {
                challenge: randomChallenge(),
                rp: { name: "Nexa" },
                user: { id: randomChallenge(), name: "local", displayName: "local" },
                pubKeyCredParams: [{ type: "public-key", alg: -7 }],
                authenticatorSelection: { authenticatorAttachment: "platform", userVerification: "required" },
                timeout: 60000,
            },
        });
        const publicKeyCredential = credential as PublicKeyCredential | null;
        if (!publicKeyCredential) {
            throw new Error("[nexa/platform] no se pudo crear la credencial biométrica.");
        }
        storage.set(STORAGE_KEY, bufferToBase64(publicKeyCredential.rawId));
        return;
    }

    const assertion = await nav.credentials.get({
        publicKey: {
            challenge: randomChallenge(),
            allowCredentials: [{ id: base64ToBuffer(storedId), type: "public-key" }],
            userVerification: "required",
            timeout: 60000,
        },
    });
    if (!assertion) {
        throw new Error("[nexa/platform] la autenticación biométrica falló.");
    }
}

function randomChallenge(): Uint8Array<ArrayBuffer> {
    const challenge = new Uint8Array(new ArrayBuffer(32));
    crypto.getRandomValues(challenge);
    return challenge;
}

function bufferToBase64(buffer: ArrayBuffer): string {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}

function base64ToBuffer(base64: string): Uint8Array<ArrayBuffer> {
    const binary = atob(base64);
    const bytes = new Uint8Array(new ArrayBuffer(binary.length));
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
}

export const biometrics = { isAvailable, authenticate };
