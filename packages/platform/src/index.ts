import { isCapacitor, isTauri, isWeb } from "./environment";
import { capturePhoto } from "./camera";
import { openCache } from "./cache";
import { openCollection } from "./db";
import { notify } from "./notifications";
import { share } from "./share";
import { createSessionStorage, createStorage } from "./storage";

export { isTauri, isCapacitor, isWeb, currentGlobal, capacitorPlugin } from "./environment";
export type { NexaPlatformGlobal } from "./environment";
export { createStorage, createSessionStorage } from "./storage";
export type { KeyValueStorage } from "./storage";
export { notify } from "./notifications";
export type { NotifyOptions, NotifyDeps, CapacitorNotificationsPlugin } from "./notifications";
export { share } from "./share";
export type { ShareOptions, ShareDeps, CapacitorSharePlugin } from "./share";
export { capturePhoto } from "./camera";
export type { CameraResult, CameraDeps, CapacitorCameraPlugin, WebCapture } from "./camera";
export { openCache } from "./cache";
export type { CacheStore } from "./cache";
export { openCollection } from "./db";
export type { Collection } from "./db";

/**
 * El objeto que de verdad se usa desde un handler de una página Nexa
 * (`platform.share(...)`, `platform.notify(...)`) — `nexa-activation`
 * (Fase 13) detecta el identificador `platform.` en el código fuente de
 * un handler y antepone `import { platform } from
 * "/assets/nexa-platform.js";` automáticamente, así que este export es
 * el contrato real con el compilador, no solo un atajo de conveniencia.
 */
export const platform = {
    isTauri,
    isCapacitor,
    isWeb,
    notify,
    share,
    capturePhoto,
    storage: createStorage,
    sessionStorage: createSessionStorage,
    cache: openCache,
    db: openCollection,
};
