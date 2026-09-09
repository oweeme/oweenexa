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

// packages/platform/src/geolocation.ts
async function getCurrentPosition(options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Geolocation");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/geolocation no est\xE1 instalado en esta app.");
    }
    return plugin.getCurrentPosition(options);
  }
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav?.geolocation) {
    throw new Error("[nexa/platform] la Geolocation API no est\xE1 disponible en este entorno.");
  }
  return new Promise((resolve, reject) => {
    nav.geolocation.getCurrentPosition(
      (position) => resolve(position),
      (err) => reject(new Error(`[nexa/platform] no se pudo obtener la posici\xF3n: ${err.message}`)),
      options
    );
  });
}
function watchPosition(callback, options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Geolocation");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/geolocation no est\xE1 instalado en esta app.");
    }
    let cancelled = false;
    let watchId;
    plugin.watchPosition(options, (position) => {
      if (position) callback(position);
    }).then((id2) => {
      if (cancelled) {
        void plugin.clearWatch({ id: id2 });
      } else {
        watchId = id2;
      }
    });
    return () => {
      cancelled = true;
      if (watchId) void plugin.clearWatch({ id: watchId });
    };
  }
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav?.geolocation) {
    throw new Error("[nexa/platform] la Geolocation API no est\xE1 disponible en este entorno.");
  }
  const id = nav.geolocation.watchPosition((position) => callback(position), void 0, options);
  return () => nav.geolocation.clearWatch(id);
}
var geolocation = { getCurrentPosition, watchPosition };

// packages/platform/src/deeplinks.ts
async function getLaunchUrl(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "App");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/app no est\xE1 instalado en esta app.");
    }
    const result = await plugin.getLaunchUrl();
    return result ?? { url: "" };
  }
  return { url: "" };
}
function onOpen(callback, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "App");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/app no est\xE1 instalado en esta app.");
    }
    let cancelled = false;
    let handle;
    plugin.addListener("appUrlOpen", callback).then((h) => {
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
  return () => {
  };
}
var deepLinks = { getLaunchUrl, onOpen };

// packages/platform/src/push.ts
async function register(options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "PushNotifications");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/push-notifications no est\xE1 instalado en esta app.");
    }
    return new Promise((resolve, reject) => {
      let registrationHandle;
      let errorHandle;
      const cleanup = () => {
        void registrationHandle?.remove();
        void errorHandle?.remove();
      };
      Promise.all([
        plugin.addListener("registration", (token) => {
          cleanup();
          resolve({ platform: "capacitor", token: token.value });
        }),
        plugin.addListener("registrationError", (error) => {
          cleanup();
          reject(new Error(`[nexa/platform] fall\xF3 el registro de push: ${error.error}`));
        })
      ]).then(([regHandle, errHandle]) => {
        registrationHandle = regHandle;
        errorHandle = errHandle;
        return plugin.register();
      }).catch(reject);
    });
  }
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav?.serviceWorker) {
    throw new Error("[nexa/platform] los Service Workers no est\xE1n disponibles en este entorno.");
  }
  if (!options.vapidPublicKey) {
    throw new Error("[nexa/platform] platform.push.register() necesita `vapidPublicKey` en la Web.");
  }
  const registration = deps.serviceWorkerRegistration ?? await nav.serviceWorker.getRegistration();
  if (!registration) {
    throw new Error(
      "[nexa/platform] no hay un Service Worker registrado \u2014 declar\xE1 [pwa] en nexa.toml (Fase 18) o registr\xE1 uno propio antes de llamar a esto."
    );
  }
  const subscription = await registration.pushManager.subscribe({
    userVisibleOnly: true,
    applicationServerKey: urlBase64ToUint8Array(options.vapidPublicKey)
  });
  return { platform: "web", subscription: subscription.toJSON() };
}
function onReceived(callback, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "PushNotifications");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/push-notifications no est\xE1 instalado en esta app.");
    }
    let cancelled = false;
    let handle;
    plugin.addListener("pushNotificationReceived", callback).then((h) => {
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
  throw new Error(
    '[nexa/platform] en la Web, un push llega al Service Worker, no a la p\xE1gina \u2014 agreg\xE1 tu propio self.addEventListener("push", ...) ah\xED.'
  );
}
function onActionPerformed(callback, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "PushNotifications");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/push-notifications no est\xE1 instalado en esta app.");
    }
    let cancelled = false;
    let handle;
    plugin.addListener("pushNotificationActionPerformed", callback).then((h) => {
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
  throw new Error(
    '[nexa/platform] en la Web, la acci\xF3n sobre un push se maneja en el Service Worker (notificationclick), no en la p\xE1gina \u2014 agreg\xE1 tu propio self.addEventListener("notificationclick", ...) ah\xED.'
  );
}
function urlBase64ToUint8Array(base64String) {
  const padding = "=".repeat((4 - base64String.length % 4) % 4);
  const base64 = (base64String + padding).replace(/-/g, "+").replace(/_/g, "/");
  const rawData = atob(base64);
  const outputArray = new Uint8Array(new ArrayBuffer(rawData.length));
  for (let i = 0; i < rawData.length; i++) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}
var push = { register, onReceived, onActionPerformed };

// packages/platform/src/biometrics.ts
var STORAGE_KEY2 = "nexa-biometrics-credential-id";
function mapCapacitorBiometryType(value) {
  switch (value) {
    case 1:
      return "touchId";
    case 2:
      return "faceId";
    case 3:
      return "fingerprint";
    case 4:
      return "face";
    case 5:
      return "iris";
    default:
      return "none";
  }
}
async function isAvailable(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "BiometricAuth");
    if (!plugin) {
      throw new Error("[nexa/platform] @aparajita/capacitor-biometric-auth no est\xE1 instalado en esta app.");
    }
    const result = await plugin.checkBiometry();
    return { isAvailable: result.isAvailable, biometryType: mapCapacitorBiometryType(result.biometryType) };
  }
  const win = deps.windowObject ?? (typeof window !== "undefined" ? window : void 0);
  const ctor = win?.PublicKeyCredential;
  if (!ctor?.isUserVerifyingPlatformAuthenticatorAvailable) {
    return { isAvailable: false, biometryType: "none" };
  }
  const available = await ctor.isUserVerifyingPlatformAuthenticatorAvailable();
  return { isAvailable: available, biometryType: available ? "platform" : "none" };
}
async function authenticate(options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "BiometricAuth");
    if (!plugin) {
      throw new Error("[nexa/platform] @aparajita/capacitor-biometric-auth no est\xE1 instalado en esta app.");
    }
    await plugin.authenticate({ reason: options.reason });
    return;
  }
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav?.credentials) {
    throw new Error("[nexa/platform] WebAuthn no est\xE1 disponible en este entorno.");
  }
  const storage = deps.storage ?? createStorage();
  const storedId = storage.get(STORAGE_KEY2);
  if (!storedId) {
    const credential = await nav.credentials.create({
      publicKey: {
        challenge: randomChallenge(),
        rp: { name: "Nexa" },
        user: { id: randomChallenge(), name: "local", displayName: "local" },
        pubKeyCredParams: [{ type: "public-key", alg: -7 }],
        authenticatorSelection: { authenticatorAttachment: "platform", userVerification: "required" },
        timeout: 6e4
      }
    });
    const publicKeyCredential = credential;
    if (!publicKeyCredential) {
      throw new Error("[nexa/platform] no se pudo crear la credencial biom\xE9trica.");
    }
    storage.set(STORAGE_KEY2, bufferToBase64(publicKeyCredential.rawId));
    return;
  }
  const assertion = await nav.credentials.get({
    publicKey: {
      challenge: randomChallenge(),
      allowCredentials: [{ id: base64ToBuffer(storedId), type: "public-key" }],
      userVerification: "required",
      timeout: 6e4
    }
  });
  if (!assertion) {
    throw new Error("[nexa/platform] la autenticaci\xF3n biom\xE9trica fall\xF3.");
  }
}
function randomChallenge() {
  const challenge = new Uint8Array(new ArrayBuffer(32));
  crypto.getRandomValues(challenge);
  return challenge;
}
function bufferToBase64(buffer) {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)));
}
function base64ToBuffer(base64) {
  const binary = atob(base64);
  const bytes = new Uint8Array(new ArrayBuffer(binary.length));
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
var biometrics = { isAvailable, authenticate };

// packages/platform/src/haptics.ts
async function impact(options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Haptics");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/haptics no est\xE1 instalado en esta app.");
    }
    await plugin.impact(options);
    return;
  }
  vibrateWithPattern(patternForImpact(options.style), deps);
}
async function notification(options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Haptics");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/haptics no est\xE1 instalado en esta app.");
    }
    await plugin.notification(options);
    return;
  }
  vibrateWithPattern(patternForNotification(options.type), deps);
}
async function vibrate(options = {}, deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Haptics");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/haptics no est\xE1 instalado en esta app.");
    }
    await plugin.vibrate(options);
    return;
  }
  vibrateWithPattern([options.duration ?? 300], deps);
}
var selectionStarted = false;
async function selectionStart(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Haptics");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/haptics no est\xE1 instalado en esta app.");
    }
    await plugin.selectionStart();
    return;
  }
  selectionStarted = true;
}
async function selectionChanged(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Haptics");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/haptics no est\xE1 instalado en esta app.");
    }
    await plugin.selectionChanged();
    return;
  }
  if (selectionStarted) {
    vibrateWithPattern([70], deps);
  }
}
async function selectionEnd(deps = {}) {
  const environment = deps.environment ?? currentGlobal();
  if (isCapacitor(environment)) {
    const plugin = deps.capacitorPlugin ?? capacitorPlugin(environment, "Haptics");
    if (!plugin) {
      throw new Error("[nexa/platform] @capacitor/haptics no est\xE1 instalado en esta app.");
    }
    await plugin.selectionEnd();
    return;
  }
  selectionStarted = false;
}
function patternForImpact(style = "HEAVY") {
  if (style === "MEDIUM") return [43];
  if (style === "LIGHT") return [20];
  return [61];
}
function patternForNotification(type = "SUCCESS") {
  if (type === "WARNING") return [30, 40, 30, 50, 60];
  if (type === "ERROR") return [27, 45, 50];
  return [35, 65, 21];
}
function vibrateWithPattern(pattern, deps) {
  const nav = deps.navigatorObject ?? (typeof navigator !== "undefined" ? navigator : void 0);
  if (!nav?.vibrate) {
    throw new Error("[nexa/platform] la Vibration API no est\xE1 disponible en este entorno.");
  }
  nav.vibrate(pattern);
}
var haptics = { impact, notification, vibrate, selectionStart, selectionChanged, selectionEnd };

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
  lifecycle,
  geolocation,
  deepLinks,
  push,
  biometrics,
  haptics
};
export {
  authenticate,
  biometrics,
  capacitorPlugin,
  capturePhoto,
  createSessionStorage,
  createStorage,
  currentGlobal,
  deepLinks,
  geolocation,
  getCurrentPosition,
  getLaunchUrl,
  getLifecycleState,
  getNetworkStatus,
  haptics,
  notification as hapticsNotification,
  impact,
  isAvailable,
  isCapacitor,
  isTauri,
  isWeb,
  lifecycle,
  network,
  notify,
  onActionPerformed,
  onLifecycleChange,
  onNetworkChange,
  onOpen,
  onReceived,
  openCache,
  openCollection,
  platform,
  push,
  register,
  selectionChanged,
  selectionEnd,
  selectionStart,
  share,
  theme,
  vibrate,
  watchPosition
};
