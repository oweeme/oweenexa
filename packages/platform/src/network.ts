import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export type ConnectionType = "wifi" | "cellular" | "none" | "unknown";

export interface ConnectionStatus {
    online: boolean;
    type: ConnectionType;
}

/** La forma real de `@capacitor/network` (`getStatus`/`addListener`/`ConnectionStatus`). */
export interface CapacitorNetworkPlugin {
    getStatus(): Promise<{ connected: boolean; connectionType: ConnectionType }>;
    addListener(
        eventName: "networkStatusChange",
        listener: (status: { connected: boolean; connectionType: ConnectionType }) => void,
    ): Promise<{ remove(): Promise<void> }>;
}

export interface NetworkDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorNetworkPlugin;
    navigatorObject?: Pick<Navigator, "onLine">;
    eventTarget?: Pick<EventTarget, "addEventListener" | "removeEventListener">;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/network`. Tauri y web:
 * `navigator.onLine` — el único dato realmente estándar; el *tipo* de
 * conexión (`wifi`/`cellular`) no tiene equivalente confiable fuera de
 * Capacitor (`navigator.connection` es una API no estándar, soportada
 * de forma inconsistente), así que la rama web siempre devuelve
 * `type: "unknown"` en vez de adivinar.
 */
export async function getNetworkStatus(deps: NetworkDeps = {}): Promise<ConnectionStatus> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorNetworkPlugin>(environment, "Network");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/network no está instalado en esta app.");
        }
        const status = await plugin.getStatus();
        return { online: status.connected, type: status.connectionType };
    }

    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav) {
        throw new Error("[nexa/platform] `navigator` no está disponible en este entorno.");
    }
    return { online: nav.onLine, type: "unknown" };
}

/**
 * Suscribe `callback` a cambios de conectividad; devuelve una función
 * para cancelar la suscripción — mismo contrato que `connectSSE`/
 * `connectSocket` de `@nexa/http` (Fase 26). Capacitor: `addListener`
 * es async (devuelve una promesa de `PluginListenerHandle`); si se
 * cancela antes de que resuelva, se remueve apenas llega en vez de
 * dejarlo huérfano. Web: los eventos estándar `online`/`offline`.
 */
export function onNetworkChange(callback: (status: ConnectionStatus) => void, deps: NetworkDeps = {}): () => void {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorNetworkPlugin>(environment, "Network");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/network no está instalado en esta app.");
        }

        let cancelled = false;
        let handle: { remove(): Promise<void> } | undefined;
        plugin
            .addListener("networkStatusChange", (status) => callback({ online: status.connected, type: status.connectionType }))
            .then((h) => {
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

    const target = deps.eventTarget ?? (typeof window !== "undefined" ? window : undefined);
    if (!target) {
        throw new Error("[nexa/platform] no hay un target de eventos disponible en este entorno.");
    }

    const onOnline = () => callback({ online: true, type: "unknown" });
    const onOffline = () => callback({ online: false, type: "unknown" });
    target.addEventListener("online", onOnline);
    target.addEventListener("offline", onOffline);

    return () => {
        target.removeEventListener("online", onOnline);
        target.removeEventListener("offline", onOffline);
    };
}

export const network = { getStatus: getNetworkStatus, onChange: onNetworkChange };
