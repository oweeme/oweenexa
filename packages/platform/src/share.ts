import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export interface ShareOptions {
    title?: string;
    text?: string;
    url?: string;
}

/** La forma real de `share()` en `@capacitor/share`. */
export interface CapacitorSharePlugin {
    share(options: ShareOptions): Promise<void>;
}

export interface ShareDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorSharePlugin;
    navigatorObject?: Pick<Navigator, "share">;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/share`. Tauri y web: la
 * Web Share API estándar (`navigator.share`) — soportada por los tres
 * motores de webview de escritorio de Tauri en sistemas actualizados;
 * donde no lo esté, esto falla con un error explícito, nunca en
 * silencio.
 */
export async function share(options: ShareOptions, deps: ShareDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorSharePlugin>(environment, "Share");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/share no está instalado en esta app.");
        }
        await plugin.share(options);
        return;
    }

    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav?.share) {
        throw new Error("[nexa/platform] la Web Share API no está disponible en este entorno.");
    }
    await nav.share(options);
}
