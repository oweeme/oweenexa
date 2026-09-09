import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export interface LifecycleState {
    /** `false` mientras la app está en segundo plano/la pestaña está oculta. */
    active: boolean;
}

/** La forma real (recortada) de `getState`/`addListener("appStateChange", ...)` en `@capacitor/app`. */
export interface CapacitorAppPlugin {
    getState(): Promise<{ isActive: boolean }>;
    addListener(
        eventName: "appStateChange",
        listener: (state: { isActive: boolean }) => void,
    ): Promise<{ remove(): Promise<void> }>;
}

export interface LifecycleDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorAppPlugin;
    documentObject?: Pick<Document, "hidden" | "addEventListener" | "removeEventListener">;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/app` (`getState`,
 * `{ isActive }`). Web/Tauri: `document.hidden` — la propia
 * documentación de `@capacitor/app` dice que su rama web ya se basa en
 * `visibilitychange`/`document.hidden`, así que replicar esa misma
 * señal directamente es fiel al comportamiento real del plugin, no una
 * aproximación propia.
 */
export async function getLifecycleState(deps: LifecycleDeps = {}): Promise<LifecycleState> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorAppPlugin>(environment, "App");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/app no está instalado en esta app.");
        }
        const state = await plugin.getState();
        return { active: state.isActive };
    }

    const doc = deps.documentObject ?? (typeof document !== "undefined" ? document : undefined);
    if (!doc) {
        throw new Error("[nexa/platform] `document` no está disponible en este entorno.");
    }
    return { active: !doc.hidden };
}

/**
 * Suscribe `callback` a cambios de ciclo de vida (pasa a segundo plano
 * / vuelve a primer plano); devuelve una función para cancelar la
 * suscripción — mismo contrato que `platform.network.onChange` (Fase
 * 48) y `connectSSE`/`connectSocket` de `@nexa/http` (Fase 26).
 */
export function onLifecycleChange(callback: (state: LifecycleState) => void, deps: LifecycleDeps = {}): () => void {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorAppPlugin>(environment, "App");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/app no está instalado en esta app.");
        }

        let cancelled = false;
        let handle: { remove(): Promise<void> } | undefined;
        plugin.addListener("appStateChange", (state) => callback({ active: state.isActive })).then((h) => {
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

    const doc = deps.documentObject ?? (typeof document !== "undefined" ? document : undefined);
    if (!doc) {
        throw new Error("[nexa/platform] `document` no está disponible en este entorno.");
    }

    const onVisibilityChange = () => callback({ active: !doc.hidden });
    doc.addEventListener("visibilitychange", onVisibilityChange);

    return () => {
        doc.removeEventListener("visibilitychange", onVisibilityChange);
    };
}

export const lifecycle = { getState: getLifecycleState, onChange: onLifecycleChange };
