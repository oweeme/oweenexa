import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export interface Coordinates {
    latitude: number;
    longitude: number;
    accuracy: number;
    altitude: number | null;
    altitudeAccuracy: number | null | undefined;
    heading: number | null;
    speed: number | null;
}

export interface Position {
    coords: Coordinates;
    timestamp: number;
}

export interface GeolocationOptions {
    enableHighAccuracy?: boolean;
    timeout?: number;
    maximumAge?: number;
}

/** La forma real (recortada) de `@capacitor/geolocation`. */
export interface CapacitorGeolocationPlugin {
    getCurrentPosition(options?: GeolocationOptions): Promise<Position>;
    watchPosition(
        options: GeolocationOptions,
        callback: (position: Position | null, err?: unknown) => void,
    ): Promise<string>;
    clearWatch(options: { id: string }): Promise<void>;
}

export interface GeolocationDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorGeolocationPlugin;
    navigatorObject?: Pick<Navigator, "geolocation">;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/geolocation`. Web/Tauri:
 * la Geolocation API estándar del navegador (`navigator.geolocation`) —
 * la misma que usa la propia rama web de `@capacitor/geolocation` por
 * dentro. La forma de `Position` coincide en ambos casos
 * (`coords.{latitude,longitude,accuracy,...}` + `timestamp`), así que
 * no hace falta normalizar nada entre ramas.
 */
export async function getCurrentPosition(options: GeolocationOptions = {}, deps: GeolocationDeps = {}): Promise<Position> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorGeolocationPlugin>(environment, "Geolocation");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/geolocation no está instalado en esta app.");
        }
        return plugin.getCurrentPosition(options);
    }

    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav?.geolocation) {
        throw new Error("[nexa/platform] la Geolocation API no está disponible en este entorno.");
    }

    return new Promise((resolve, reject) => {
        nav.geolocation.getCurrentPosition(
            (position) => resolve(position),
            (err) => reject(new Error(`[nexa/platform] no se pudo obtener la posición: ${err.message}`)),
            options,
        );
    });
}

/**
 * Suscribe `callback` a cambios de posición; devuelve una función para
 * cancelar la suscripción — mismo contrato que `platform.network.onChange`/
 * `platform.lifecycle.onChange`. Capacitor: `watchPosition` es async
 * (devuelve una promesa del id de watch); si se cancela antes de que
 * resuelva, se limpia apenas llega en vez de dejarlo huérfano.
 */
export function watchPosition(
    callback: (position: Position) => void,
    options: GeolocationOptions = {},
    deps: GeolocationDeps = {},
): () => void {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorGeolocationPlugin>(environment, "Geolocation");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/geolocation no está instalado en esta app.");
        }

        let cancelled = false;
        let watchId: string | undefined;
        plugin
            .watchPosition(options, (position) => {
                if (position) callback(position);
            })
            .then((id) => {
                if (cancelled) {
                    void plugin.clearWatch({ id });
                } else {
                    watchId = id;
                }
            });

        return () => {
            cancelled = true;
            if (watchId) void plugin.clearWatch({ id: watchId });
        };
    }

    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav?.geolocation) {
        throw new Error("[nexa/platform] la Geolocation API no está disponible en este entorno.");
    }

    const id = nav.geolocation.watchPosition((position) => callback(position), undefined, options);
    return () => nav.geolocation.clearWatch(id);
}

export const geolocation = { getCurrentPosition, watchPosition };
