/**
 * Lo mínimo que hace falta leer de `window`/`globalThis` para saber en
 * qué entorno se está corriendo. Las formas exactas están verificadas
 * contra el código fuente real de `@tauri-apps/api` (`isTauri()` mira
 * `globalThis.isTauri`) y `@capacitor/core` (`window.Capacitor.
 * isNativePlatform()`), no inventadas.
 */
export interface NexaPlatformGlobal {
    isTauri?: boolean;
    Capacitor?: {
        isNativePlatform?: () => boolean;
        Plugins?: Record<string, Record<string, (...args: never[]) => unknown>>;
    };
}

export function currentGlobal(): NexaPlatformGlobal {
    if (typeof window !== "undefined") {
        return window as unknown as NexaPlatformGlobal;
    }
    if (typeof globalThis !== "undefined") {
        return globalThis as unknown as NexaPlatformGlobal;
    }
    return {};
}

export function isTauri(target: NexaPlatformGlobal = currentGlobal()): boolean {
    return !!target.isTauri;
}

export function isCapacitor(target: NexaPlatformGlobal = currentGlobal()): boolean {
    return !!target.Capacitor?.isNativePlatform?.();
}

export function isWeb(target: NexaPlatformGlobal = currentGlobal()): boolean {
    return !isTauri(target) && !isCapacitor(target);
}

/** Un plugin de Capacitor por nombre (`window.Capacitor.Plugins.X`), si existe. */
export function capacitorPlugin<T>(target: NexaPlatformGlobal, name: string): T | undefined {
    return target.Capacitor?.Plugins?.[name] as T | undefined;
}
