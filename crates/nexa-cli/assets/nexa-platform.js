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
function createStorage(backend) {
  const store = backend ?? (typeof localStorage !== "undefined" ? localStorage : void 0);
  if (!store) {
    throw new Error("[nexa/platform] no hay almacenamiento disponible en este entorno.");
  }
  return {
    get: (key) => store.getItem(key),
    set: (key, value) => store.setItem(key, value),
    remove: (key) => store.removeItem(key)
  };
}

// packages/platform/src/index.ts
var platform = {
  isTauri,
  isCapacitor,
  isWeb,
  notify,
  share,
  capturePhoto,
  storage: createStorage
};
export {
  capacitorPlugin,
  capturePhoto,
  createStorage,
  currentGlobal,
  isCapacitor,
  isTauri,
  isWeb,
  notify,
  platform,
  share
};
