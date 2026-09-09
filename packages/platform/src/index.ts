import { isCapacitor, isTauri, isWeb } from "./environment";
import { capturePhoto } from "./camera";
import { openCache } from "./cache";
import { openCollection } from "./db";
import { notify } from "./notifications";
import { share } from "./share";
import { createSessionStorage, createStorage } from "./storage";
import { theme } from "./theme";
import { network } from "./network";
import { lifecycle } from "./lifecycle";
import { geolocation } from "./geolocation";
import { deepLinks } from "./deeplinks";
import { push } from "./push";

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
export { theme } from "./theme";
export type { Theme } from "./theme";
export { getNetworkStatus, onNetworkChange, network } from "./network";
export type { ConnectionStatus, ConnectionType, NetworkDeps, CapacitorNetworkPlugin } from "./network";
export { getLifecycleState, onLifecycleChange, lifecycle } from "./lifecycle";
export type { LifecycleState, LifecycleDeps, CapacitorAppPlugin } from "./lifecycle";
export { getCurrentPosition, watchPosition, geolocation } from "./geolocation";
export type { Position, Coordinates, GeolocationOptions, GeolocationDeps, CapacitorGeolocationPlugin } from "./geolocation";
export { getLaunchUrl, onOpen, deepLinks } from "./deeplinks";
export type { LaunchUrl, AppUrlOpenEvent, DeepLinksDeps, CapacitorAppLinksPlugin } from "./deeplinks";
export { register, onReceived, onActionPerformed, push } from "./push";
export type { PushRegistration, PushOptions, PushDeps, CapacitorPushPlugin } from "./push";

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
    theme,
    network,
    lifecycle,
    geolocation,
    deepLinks,
    push,
};
