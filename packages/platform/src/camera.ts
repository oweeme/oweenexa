import { capacitorPlugin, currentGlobal, isCapacitor, type NexaPlatformGlobal } from "./environment";

export interface CameraResult {
    /** Un `data:image/...;base64,...` listo para usar en `<img src>`. */
    dataUrl: string;
}

/** La forma real (recortada) de `getPhoto()` en `@capacitor/camera`. */
export interface CapacitorCameraPlugin {
    getPhoto(options: { resultType: "dataUrl"; quality?: number }): Promise<{ dataUrl?: string }>;
}

export type WebCapture = () => Promise<CameraResult>;

export interface CameraDeps {
    environment?: NexaPlatformGlobal;
    capacitorPlugin?: CapacitorCameraPlugin;
    webCapture?: WebCapture;
}

/**
 * Capacitor nativo: el plugin real `@capacitor/camera`, pidiendo
 * directamente un `dataUrl` (evita depender de rutas de archivo
 * (`webPath`) que no tienen sentido fuera de una app nativa).
 *
 * Tauri y web: no hay una forma de "tomar una foto" de una sola llamada
 * en la Web Platform — hace falta pedir un `MediaStream`
 * (`getUserMedia`), dibujar un frame en un `<canvas>` oculto, y leerlo
 * como `dataUrl`. Esa pieza (`defaultWebCapture`) es la única de todo
 * este paquete que no se puede probar con un DOM de pruebas (no hay
 * cámara real en CI) — por eso es inyectable (`webCapture`), y lo que sí
 * se prueba es que la rama correcta se elige según el entorno.
 */
export async function capturePhoto(deps: CameraDeps = {}): Promise<CameraResult> {
    const environment = deps.environment ?? currentGlobal();

    if (isCapacitor(environment)) {
        const plugin = deps.capacitorPlugin ?? capacitorPlugin<CapacitorCameraPlugin>(environment, "Camera");
        if (!plugin) {
            throw new Error("[nexa/platform] @capacitor/camera no está instalado en esta app.");
        }
        const photo = await plugin.getPhoto({ resultType: "dataUrl", quality: 90 });
        if (!photo.dataUrl) {
            throw new Error("[nexa/platform] la cámara no devolvió ninguna imagen.");
        }
        return { dataUrl: photo.dataUrl };
    }

    const capture = deps.webCapture ?? defaultWebCapture;
    return capture();
}

async function defaultWebCapture(): Promise<CameraResult> {
    if (typeof navigator === "undefined" || !navigator.mediaDevices?.getUserMedia) {
        throw new Error("[nexa/platform] getUserMedia no está disponible en este entorno.");
    }

    const stream = await navigator.mediaDevices.getUserMedia({ video: true });
    try {
        const video = document.createElement("video");
        video.srcObject = stream;
        video.muted = true;
        await video.play();

        const canvas = document.createElement("canvas");
        canvas.width = video.videoWidth;
        canvas.height = video.videoHeight;
        const ctx = canvas.getContext("2d");
        if (!ctx) {
            throw new Error("[nexa/platform] no se pudo obtener un contexto 2D de canvas.");
        }
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);

        return { dataUrl: canvas.toDataURL("image/png") };
    } finally {
        for (const track of stream.getTracks()) {
            track.stop();
        }
    }
}
