import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export interface NotifyOptions {
    title: string;
    body?: string;
}

/** La forma real de `schedule()` en `@capacitor/local-notifications`. */
export interface CapacitorNotificationsPlugin {
    schedule(options: {
        notifications: Array<{ id: number; title: string; body: string }>;
    }): Promise<void>;
}

export interface NotifyDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorNotificationsPlugin;
    notificationCtor?: typeof Notification;
}

/**
 * Capacitor nativo: usa el plugin real `@capacitor/local-notifications`
 * (el usuario debe instalarlo en su app Capacitor; aquí solo se llama a
 * través del puente ya expuesto en `window.Capacitor.Plugins`).
 *
 * Tauri y web: la Notification API estándar del navegador — el webview
 * de Tauri la implementa igual que un navegador real, así que no hace
 * falta un plugin dedicado para este caso básico (queda como posible
 * mejora, no como algo que esta fase deje a medias: sin plugin, Tauri
 * necesita que el sistema operativo ya tenga el permiso concedido).
 */
export async function notify(options: NotifyOptions, deps: NotifyDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorNotificationsPlugin>(environment, "LocalNotifications");
        if (!plugin) {
            throw new Error(
                "[nexa/platform] @capacitor/local-notifications no está instalado en esta app.",
            );
        }
        await plugin.schedule({
            notifications: [
                { id: Date.now() % 2147483647, title: options.title, body: options.body ?? "" },
            ],
        });
        return;
    }

    const Ctor = deps.notificationCtor ?? (typeof Notification !== "undefined" ? Notification : undefined);
    if (!Ctor) {
        throw new Error("[nexa/platform] la Notification API no está disponible en este entorno.");
    }

    const permission = Ctor.permission === "granted" ? "granted" : await Ctor.requestPermission();
    if (permission !== "granted") {
        throw new Error("[nexa/platform] permiso de notificaciones denegado.");
    }

    new Ctor(options.title, { body: options.body });
}
