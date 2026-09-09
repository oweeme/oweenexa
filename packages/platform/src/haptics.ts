import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export type ImpactStyle = "HEAVY" | "MEDIUM" | "LIGHT";
export type HapticsNotificationType = "SUCCESS" | "WARNING" | "ERROR";

export interface ImpactOptions {
    style?: ImpactStyle;
}

export interface HapticsNotificationOptions {
    type?: HapticsNotificationType;
}

export interface VibrateOptions {
    duration?: number;
}

/** La forma real de `@capacitor/haptics`. */
export interface CapacitorHapticsPlugin {
    impact(options?: ImpactOptions): Promise<void>;
    notification(options?: HapticsNotificationOptions): Promise<void>;
    vibrate(options?: VibrateOptions): Promise<void>;
    selectionStart(): Promise<void>;
    selectionChanged(): Promise<void>;
    selectionEnd(): Promise<void>;
}

export interface HapticsDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorHapticsPlugin;
    navigatorObject?: Pick<Navigator, "vibrate">;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/haptics`. Web/Tauri: la
 * Vibration API estándar del navegador (`navigator.vibrate`) — la
 * misma API que usa la propia rama web de `@capacitor/haptics` por
 * dentro (a diferencia de biometría, esta rama web SÍ es real, no una
 * simulación). Los patrones de vibración de cada estilo/tipo son
 * exactamente los que usa `HapticsWeb`, verificados leyendo su código
 * fuente — no inventados.
 */
export async function impact(options: ImpactOptions = {}, deps: HapticsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorHapticsPlugin>(environment, "Haptics");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/haptics no está instalado en esta app.");
        }
        await plugin.impact(options);
        return;
    }

    vibrateWithPattern(patternForImpact(options.style), deps);
}

export async function notification(options: HapticsNotificationOptions = {}, deps: HapticsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorHapticsPlugin>(environment, "Haptics");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/haptics no está instalado en esta app.");
        }
        await plugin.notification(options);
        return;
    }

    vibrateWithPattern(patternForNotification(options.type), deps);
}

export async function vibrate(options: VibrateOptions = {}, deps: HapticsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorHapticsPlugin>(environment, "Haptics");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/haptics no está instalado en esta app.");
        }
        await plugin.vibrate(options);
        return;
    }

    vibrateWithPattern([options.duration ?? 300], deps);
}

/**
 * `selectionStart`/`selectionChanged`/`selectionEnd`: el hint háptico
 * de "arrastrar por una selección" (ej. un date picker, un slider) —
 * `selectionChanged()` solo vibra si `selectionStart()` se llamó antes
 * y no se llamó `selectionEnd()` todavía. Mismo estado que mantiene
 * `HapticsWeb` en su propia instancia; acá es una variable de módulo,
 * fiel a que solo hay una "instancia" real por página.
 */
let selectionStarted = false;

export async function selectionStart(deps: HapticsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorHapticsPlugin>(environment, "Haptics");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/haptics no está instalado en esta app.");
        }
        await plugin.selectionStart();
        return;
    }

    selectionStarted = true;
}

export async function selectionChanged(deps: HapticsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorHapticsPlugin>(environment, "Haptics");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/haptics no está instalado en esta app.");
        }
        await plugin.selectionChanged();
        return;
    }

    if (selectionStarted) {
        vibrateWithPattern([70], deps);
    }
}

export async function selectionEnd(deps: HapticsDeps = {}): Promise<void> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorHapticsPlugin>(environment, "Haptics");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/haptics no está instalado en esta app.");
        }
        await plugin.selectionEnd();
        return;
    }

    selectionStarted = false;
}

function patternForImpact(style: ImpactStyle = "HEAVY"): number[] {
    if (style === "MEDIUM") return [43];
    if (style === "LIGHT") return [20];
    return [61];
}

function patternForNotification(type: HapticsNotificationType = "SUCCESS"): number[] {
    if (type === "WARNING") return [30, 40, 30, 50, 60];
    if (type === "ERROR") return [27, 45, 50];
    return [35, 65, 21];
}

function vibrateWithPattern(pattern: number[], deps: HapticsDeps): void {
    const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : undefined);
    if (!nav?.vibrate) {
        throw new Error("[nexa/platform] la Vibration API no está disponible en este entorno.");
    }
    nav.vibrate(pattern);
}

export const haptics = { impact, notification, vibrate, selectionStart, selectionChanged, selectionEnd };
