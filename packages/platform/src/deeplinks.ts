import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export interface LaunchUrl {
    url: string;
}

export interface AppUrlOpenEvent {
    url: string;
}

/** La forma real (recortada) de `getLaunchUrl`/`addListener("appUrlOpen", ...)` en `@capacitor/app`. */
export interface CapacitorAppLinksPlugin {
    getLaunchUrl(): Promise<LaunchUrl | undefined>;
    addListener(
        eventName: "appUrlOpen",
        listener: (event: AppUrlOpenEvent) => void,
    ): Promise<{ remove(): Promise<void> }>;
}

export interface DeepLinksDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorAppLinksPlugin;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/app` (`getLaunchUrl`).
 * Web/Tauri: siempre `{ url: "" }` — no una aproximación propia, es
 * literalmente lo que hace la propia rama web de `@capacitor/app`
 * (`AppWeb.getLaunchUrl` devuelve `{ url: '' }` sin condición, ver su
 * código fuente). Tiene sentido: una página no se "lanza" con una URL
 * de deep link distinta de la que ya está cargada, a diferencia de una
 * app nativa abierta vía un esquema de URL personalizado.
 */
export async function getLaunchUrl(deps: DeepLinksDeps = {}): Promise<LaunchUrl> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorAppLinksPlugin>(environment, "App");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/app no está instalado en esta app.");
        }
        const result = await plugin.getLaunchUrl();
        return result ?? { url: "" };
    }

    return { url: "" };
}

/**
 * Suscribe `callback` a que la app se reabra con una URL de deep link
 * mientras ya está corriendo — mismo contrato de cancelación que
 * `platform.network.onChange`/`platform.lifecycle.onChange`. Capacitor:
 * `addListener("appUrlOpen", ...)` real. Web/Tauri: no existe un
 * evento equivalente — cada URL nueva en un navegador es una
 * navegación distinta, no una app ya corriendo que "se reabre"; la
 * propia `AppWeb` nunca notifica este evento en su rama web. La
 * función de cancelación que devuelve no hace nada, porque no hay
 * nada que cancelar.
 */
export function onOpen(callback: (event: AppUrlOpenEvent) => void, deps: DeepLinksDeps = {}): () => void {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorAppLinksPlugin>(environment, "App");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/app no está instalado en esta app.");
        }

        let cancelled = false;
        let handle: { remove(): Promise<void> } | undefined;
        plugin.addListener("appUrlOpen", callback).then((h) => {
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

    return () => {};
}

export const deepLinks = { getLaunchUrl, onOpen };
