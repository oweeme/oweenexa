import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

/**
 * Push real (issue #20) — distinto de `platform.notify()` (Fase 5,
 * notificación local, sin servidor de por medio). Esto es
 * deliberadamente más chico que el plugin completo de Capacitor: solo
 * registro + recibir — sin `checkPermissions`/`requestPermissions`
 * separados, sin canales de Android, sin gestión de notificaciones ya
 * entregadas. Si un proyecto real necesita eso, es la próxima pieza a
 * agregar, no algo que haya que resolver de antemano.
 *
 * Capacitor y Web no tienen la misma forma de "registro" — un token de
 * FCM/APNs no es lo mismo que una `PushSubscription` del Web Push
 * estándar — así que `PushRegistration` es una unión discriminada en
 * vez de forzar una forma común inventada.
 */
export type PushRegistration =
    | { platform: "capacitor"; token: string }
    | { platform: "web"; subscription: PushSubscriptionJSON };

export interface PushOptions {
    /** Necesaria en Web (la VAPID public key de tu propio backend) — ignorada en Capacitor. */
    vapidPublicKey?: string;
}

/** La forma real (recortada) de `@capacitor/push-notifications`. */
export interface CapacitorPushPlugin {
    register(): Promise<void>;
    addListener(eventName: "registration", listener: (token: { value: string }) => void): Promise<{ remove(): Promise<void> }>;
    addListener(
        eventName: "registrationError",
        listener: (error: { error: string }) => void,
    ): Promise<{ remove(): Promise<void> }>;
    addListener(
        eventName: "pushNotificationReceived",
        listener: (notification: unknown) => void,
    ): Promise<{ remove(): Promise<void> }>;
    addListener(
        eventName: "pushNotificationActionPerformed",
        listener: (action: unknown) => void,
    ): Promise<{ remove(): Promise<void> }>;
}

export interface PushDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorPushPlugin;
    navigatorObject?: Pick<Navigator, "serviceWorker">;
    /** Inyectable para tests — evita depender de un Service Worker real registrado. */
    serviceWorkerRegistration?: ServiceWorkerRegistration;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/push-notifications`
 * (`register()` + el evento `registration`/`registrationError` —
 * envuelto en una única `Promise`, más cómodo que suscribirse a mano).
 * Web: el Web Push estándar (`PushManager.subscribe`) — **no** es una
 * aproximación de `@capacitor/push-notifications`, que no tiene rama
 * web en absoluto (no existe un `web.js` en el paquete, verificado). En
 * Web hace falta un Service Worker ya registrado (`[pwa]` en
 * `nexa.toml`, Fase 18, o uno propio) y una `vapidPublicKey` real de tu
 * backend — sin eso, no hay a quién avisarle que llegó un push.
 */
export async function register(options: PushOptions = {}, deps: PushDeps = {}): Promise<PushRegistration> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorPushPlugin>(environment, "PushNotifications");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/push-notifications no está instalado en esta app.");
        }

        return new Promise((resolve, reject) => {
            let registrationHandle: { remove(): Promise<void> } | undefined;
            let errorHandle: { remove(): Promise<void> } | undefined;
            const cleanup = () => {
                void registrationHandle?.remove();
                void errorHandle?.remove();
            };

            Promise.all([
                plugin.addListener("registration", (token) => {
                    cleanup();
                    resolve({ platform: "capacitor", token: token.value });
                }),
                plugin.addListener("registrationError", (error) => {
                    cleanup();
                    reject(new Error(`[nexa/platform] falló el registro de push: ${error.error}`));
                }),
            ])
                .then(([regHandle, errHandle]) => {
                    registrationHandle = regHandle;
                    errorHandle = errHandle;
                    return plugin.register();
                })
                .catch(reject);
        });
    }

    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav?.serviceWorker) {
        throw new Error("[nexa/platform] los Service Workers no están disponibles en este entorno.");
    }
    if (!options.vapidPublicKey) {
        throw new Error("[nexa/platform] platform.push.register() necesita `vapidPublicKey` en la Web.");
    }

    const registration = deps.serviceWorkerRegistration ?? (await nav.serviceWorker.getRegistration());
    if (!registration) {
        throw new Error(
            "[nexa/platform] no hay un Service Worker registrado — declará [pwa] en nexa.toml (Fase 18) o registrá uno propio antes de llamar a esto.",
        );
    }

    const subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(options.vapidPublicKey),
    });

    return { platform: "web", subscription: subscription.toJSON() as PushSubscriptionJSON };
}

/**
 * Suscribe `callback` a que llegue un push mientras la app está
 * abierta. Capacitor: el evento real `pushNotificationReceived`. En
 * Web esto lanza un error explícito en vez de fallar en silencio: un
 * push en la Web **siempre** llega al Service Worker
 * (`self.addEventListener("push", ...)`), nunca a esta página — no
 * hay ninguna forma de interceptarlo desde aquí, así que fingir que
 * `onReceived` "funciona" sin hacer nada sería peor que decir
 * explícitamente por qué no.
 */
export function onReceived(callback: (notification: unknown) => void, deps: PushDeps = {}): () => void {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorPushPlugin>(environment, "PushNotifications");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/push-notifications no está instalado en esta app.");
        }

        let cancelled = false;
        let handle: { remove(): Promise<void> } | undefined;
        plugin.addListener("pushNotificationReceived", callback).then((h) => {
            if (cancelled) {
                void h.remove();
            } else {
                handle = h;
            }
        });

        return () => {
            cancelled = true;
            void handle?.remove();
        };
    }

    throw new Error(
        '[nexa/platform] en la Web, un push llega al Service Worker, no a la página — agregá tu propio self.addEventListener("push", ...) ahí.',
    );
}

/**
 * Suscribe `callback` a que el usuario interactúe con un push ya
 * entregado (lo abre, toca una acción). Mismo criterio que
 * `onReceived`: solo Capacitor tiene un evento real para esto; en Web
 * lanza el mismo error explícito, por la misma razón.
 */
export function onActionPerformed(callback: (action: unknown) => void, deps: PushDeps = {}): () => void {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorPushPlugin>(environment, "PushNotifications");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/push-notifications no está instalado en esta app.");
        }

        let cancelled = false;
        let handle: { remove(): Promise<void> } | undefined;
        plugin.addListener("pushNotificationActionPerformed", callback).then((h) => {
            if (cancelled) {
                void h.remove();
            } else {
                handle = h;
            }
        });

        return () => {
            cancelled = true;
            void handle?.remove();
        };
    }

    throw new Error(
        '[nexa/platform] en la Web, la acción sobre un push se maneja en el Service Worker (notificationclick), no en la página — agregá tu propio self.addEventListener("notificationclick", ...) ahí.',
    );
}

/** Conversión estándar de una VAPID public key en base64url a `Uint8Array` — el formato que pide `PushManager.subscribe`. */
function urlBase64ToUint8Array(base64String: string): Uint8Array<ArrayBuffer> {
    const padding = "=".repeat((4 - (base64String.length % 4)) % 4);
    const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
    const rawData = atob(base64);
    const outputArray = new Uint8Array(new ArrayBuffer(rawData.length));
    for (let i = 0; i < rawData.length; i++) {
        outputArray[i] = rawData.charCodeAt(i);
    }
    return outputArray;
}

export const push = { register, onReceived, onActionPerformed };
