// packages/platform/src/environment.ts
function currentGlobal() {
  if (typeof window !== "undefined") {
    return window;
  }
  if (typeof globalThis !== "undefined") {
    return globalThis;
  }
  return {};
}
function isTauri(target = currentGlobal()) {
  return !!target.isTauri;
}
function isCapacitor(target = currentGlobal()) {
  return !!target.Capacitor?.isNativePlatform?.();
}
function isWeb(target = currentGlobal()) {
  return !isTauri(target) && !isCapacitor(target);
}
function capacitorPlugin(target, name) {
  return target.Capacitor?.Plugins?.[name];
}

// packages/platform/src/camera.ts
async function capturePhoto(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Camera");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/camera no est\xE1 instalado en esta app.");
    }
    const photo = await plugin.getPhoto({ resultType: "dataUrl", quality: 90 });
    if (!photo.dataUrl) {
      throw new Error("[nexa/platform] la c\xE1mara no devolvi\xF3 ninguna imagen.");
    }
    return { dataUrl: photo.dataUrl };
  }
  const capture = deps.webCapture ?? defaultWebCapture;
  return capture();
}
async function defaultWebCapture() {
  if (typeof navigator === "undefined" || !navigator.mediaDevices?.getUserMedia) {
    throw new Error("[nexa/platform] getUserMedia no est\xE1 disponible en este entorno.");
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

// packages/platform/src/cache.ts
async function openCache(name, backend) {
  const storage = backend ?? (typeof caches !== "undefined" ? caches : void 0);
  if (!storage) {
    throw new Error("[nexa/platform] la Cache API no est\xE1 disponible en este entorno.");
  }
  const cache = await storage.open(name);
  return {
    match: (request) => cache.match(request),
    put: (request, response) => cache.put(request, response),
    delete: (request) => cache.delete(request)
  };
}

// packages/platform/src/db.ts
var STORE_NAME = "items";
function openDatabase(name, factory) {
  return new Promise((resolve, reject) => {
    const request = factory.open(`nexa-db-${name}`, 1);
    request.onupgradeneeded = () => {
      request.result.createObjectStore(STORE_NAME);
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}
function runRequest(request) {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}
function openCollection(name, backend) {
  const factory = backend ?? (typeof indexedDB !== "undefined" ? indexedDB : void 0);
  if (!factory) {
    throw new Error("[nexa/platform] IndexedDB no est\xE1 disponible en este entorno.");
  }
  const dbPromise = openDatabase(name, factory);
  async function withStore(mode, run) {
    const db = await dbPromise;
    const tx = db.transaction(STORE_NAME, mode);
    return runRequest(run(tx.objectStore(STORE_NAME)));
  }
  return {
    get: (id) => withStore("readonly", (store) => store.get(id)),
    set: async (id, value) => {
      await withStore("readwrite", (store) => store.put(value, id));
    },
    delete: async (id) => {
      await withStore("readwrite", (store) => store.delete(id));
    },
    list: () => withStore("readonly", (store) => store.getAll())
  };
}

// packages/platform/src/notifications.ts
async function notify(options, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "LocalNotifications");
    if (!plugin) {
      throw new Error(
        "[nexa/platform] @capacitor/local-notifications no est\xE1 instalado en esta app."
      );
    }
    await plugin.schedule({
      notifications: [
        { id: Date.now() % 2147483647, title: options.title, body: options.body ?? "" }
      ]
    });
    return;
  }
  const Ctor = deps.notificationCtor ?? (typeof Notification !== "undefined" ? Notification : void 0);
  if (!Ctor) {
    throw new Error("[nexa/platform] la Notification API no est\xE1 disponible en este entorno.");
  }
  const permission = Ctor.permission === "granted" ? "granted" : await Ctor.requestPermission();
  if (permission !== "granted") {
    throw new Error("[nexa/platform] permiso de notificaciones denegado.");
  }
  new Ctor(options.title, { body: options.body });
}

// packages/platform/src/share.ts
async function share(options, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Share");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/share no est\xE1 instalado en esta app.");
    }
    await plugin.share(options);
    return;
  }
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav?.share) {
    throw new Error("[nexa/platform] la Web Share API no est\xE1 disponible en este entorno.");
  }
  await nav.share(options);
}

// packages/platform/src/storage.ts
function wrapStorage(store) {
  return {
    get: (key) => store.getItem(key),
    set: (key, value) => store.setItem(key, value),
    remove: (key) => store.removeItem(key)
  };
}
function createStorage(backend) {
  const store = backend ?? (typeof localStorage !== "undefined" ? localStorage : void 0);
  if (!store) {
    throw new Error("[nexa/platform] no hay almacenamiento disponible en este entorno.");
  }
  return wrapStorage(store);
}
function createSessionStorage(backend) {
  const store = backend ?? (typeof sessionStorage !== "undefined" ? sessionStorage : void 0);
  if (!store) {
    throw new Error("[nexa/platform] no hay almacenamiento de sesi\xF3n disponible en este entorno.");
  }
  return wrapStorage(store);
}

// packages/platform/src/theme.ts
var STORAGE_KEY = "nexa-theme";
function applyTheme(value) {
  if (typeof document === "undefined") return;
  if (value === "system") {
    document.documentElement.removeAttribute("data-theme");
  } else {
    document.documentElement.setAttribute("data-theme", value);
  }
}
function readStoredTheme() {
  try {
    const stored = createStorage().get(STORAGE_KEY);
    return stored === "dark" || stored === "light" ? stored : "system";
  } catch {
    return "system";
  }
}
var theme = {
  get() {
    return readStoredTheme();
  },
  set(value) {
    createStorage().set(STORAGE_KEY, value);
    applyTheme(value);
  }
};
applyTheme(readStoredTheme());

// packages/platform/src/network.ts
async function getNetworkStatus(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Network");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/network no est\xE1 instalado en esta app.");
    }
    const status = await plugin.getStatus();
    return { online: status.connected, type: status.connectionType };
  }
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav) {
    throw new Error("[nexa/platform] `navigator` no est\xE1 disponible en este entorno.");
  }
  return { online: nav.onLine, type: "unknown" };
}
function onNetworkChange(callback, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Network");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/network no est\xE1 instalado en esta app.");
    }
    let cancelled = false;
    let handle;
    plugin.addListener("networkStatusChange", (status) => callback({ online: status.connected, type: status.connectionType })).then((h) => {
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
  const target = deps.eventTarget ?? (typeof window !== "undefined" ? window : void 0);
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
var network = { getStatus: getNetworkStatus, onChange: onNetworkChange };

// packages/platform/src/lifecycle.ts
async function getLifecycleState(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "App");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/app no est\xE1 instalado en esta app.");
    }
    const state = await plugin.getState();
    return { active: state.isActive };
  }
  const doc = deps.documentObject ?? (typeof document !== "undefined" ? document : void 0);
  if (!doc) {
    throw new Error("[nexa/platform] `document` no est\xE1 disponible en este entorno.");
  }
  return { active: !doc.hidden };
}
function onLifecycleChange(callback, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "App");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/app no est\xE1 instalado en esta app.");
    }
    let cancelled = false;
    let handle;
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
  const doc = deps.documentObject ?? (typeof document !== "undefined" ? document : void 0);
  if (!doc) {
    throw new Error("[nexa/platform] `document` no est\xE1 disponible en este entorno.");
  }
  const onVisibilityChange = () => callback({ active: !doc.hidden });
  doc.addEventListener("visibilitychange", onVisibilityChange);
  return () => {
    doc.removeEventListener("visibilitychange", onVisibilityChange);
  };
}
var lifecycle = { getState: getLifecycleState, onChange: onLifecycleChange };

// packages/platform/src/index.ts
var platform = {
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
  lifecycle
};
export {
  capacitorPlugin,
  capturePhoto,
  createSessionStorage,
  createStorage,
  currentGlobal,
  getLifecycleState,
  getNetworkStatus,
  isCapacitor,
  isTauri,
  isWeb,
  lifecycle,
  network,
  notify,
  onLifecycleChange,
  onNetworkChange,
  openCache,
  openCollection,
  platform,
  share,
  theme
};
