import { currentGlobal, isTauri, type NexaPlatformGlobal } from "./environment";

/**
 * Puente hacia el lado Rust de una app de escritorio (Fase 63): llama a
 * un comando de `src-tauri/src/lib.rs` (`#[tauri::command]`) y devuelve
 * su resultado ya deserializado de JSON. Fuera de Tauri (el navegador,
 * Capacitor) lanza un error claro en vez de fallar en silencio, para que
 * un código que dependa de Rust no parezca funcionar en la web.
 *
 * ```ts
 * const info = await platform.native.invoke<AppInfo>("app_info");
 * ```
 *
 * Para exponer tu propia función de Rust: agrégala a `src-tauri/src/lib.rs`
 * con `#[tauri::command]` y regístrala en `tauri::generate_handler![...]`.
 */
export async function invoke<T>(
    command: string,
    args?: Record<string, unknown>,
    target: NexaPlatformGlobal = currentGlobal(),
): Promise<T> {
    if (!isTauri(target)) {
        throw new Error(
            `platform.native.invoke("${command}") solo funciona dentro de la app de escritorio (Tauri) — estás en un navegador o en otro runtime.`,
        );
    }
    const core = target.__TAURI__?.core;
    if (!core?.invoke) {
        throw new Error(
            "Tauri no expuso `window.__TAURI__` — falta `app.withGlobalTauri: true` en src-tauri/tauri.conf.json (lo deja puesto `nexa add tauri`).",
        );
    }
    return (await core.invoke(command, args)) as T;
}

/** Lo que devuelve el comando de ejemplo `app_info` de la plantilla de `nexa add tauri`. */
export interface AppInfo {
    name: string;
    version: string;
    os: string;
    arch: string;
}

export function appInfo(target: NexaPlatformGlobal = currentGlobal()): Promise<AppInfo> {
    return invoke<AppInfo>("app_info", undefined, target);
}

export const native = { invoke, appInfo };
