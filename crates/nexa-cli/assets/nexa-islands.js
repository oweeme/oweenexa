// packages/runtime/src/strategies.ts
var interaction = (el, event, trigger) => {
  const listener = (nativeEvent) => {
    el.removeEventListener(event, listener);
    void trigger().then(() => {
      el.dispatchEvent(cloneEvent(nativeEvent));
    });
  };
  el.addEventListener(event, listener);
  return () => el.removeEventListener(event, listener);
};
var visible = (el, _event, trigger) => {
  const observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        observer.disconnect();
        void trigger();
      }
    }
  });
  observer.observe(el);
  return () => observer.disconnect();
};
var idle = (_el, _event, trigger) => {
  const requestIdle = typeof requestIdleCallback === "function" ? requestIdleCallback : (fn) => setTimeout(() => fn(makeIdleDeadline()), 1);
  const cancelIdle = typeof cancelIdleCallback === "function" ? cancelIdleCallback : (id) => clearTimeout(id);
  const handle = requestIdle(() => void trigger());
  return () => cancelIdle(handle);
};
var load = (_el, _event, trigger) => {
  void trigger();
  return () => {
  };
};
var manualTriggers = /* @__PURE__ */ new WeakMap();
var manual = (el, _event, trigger) => {
  manualTriggers.set(el, trigger);
  return () => manualTriggers.delete(el);
};
function cloneEvent(event) {
  const EventCtor = event.constructor;
  try {
    return new EventCtor(event.type, event);
  } catch {
    return new Event(event.type, { bubbles: event.bubbles, cancelable: event.cancelable });
  }
}
function makeIdleDeadline() {
  return {
    didTimeout: false,
    timeRemaining: () => 0
  };
}

// packages/islands/src/index.ts
var defaultLoader = (specifier) => import(
  /* @vite-ignore */
  specifier
);
var runners = { interaction, visible, idle, load, manual };
function initIslands(options = {}) {
  const root = options.root ?? document;
  const loadModule = options.loadModule ?? defaultLoader;
  const disposers = [];
  for (const el of Array.from(root.querySelectorAll("[data-nexa-island]"))) {
    const specifier = el.getAttribute("data-nexa-island");
    if (!specifier) continue;
    const props = parseProps(el.getAttribute("data-nexa-props"));
    const strategy = el.getAttribute("data-nexa-strategy") ?? "visible";
    const triggerEvent = strategy === "interaction" ? "pointerdown" : "";
    const trigger = async () => {
      const mod = await loadModule(specifier);
      const cleanup = mod.default(el, props);
      if (typeof cleanup === "function") disposers.push(cleanup);
    };
    const runner = runners[strategy] ?? visible;
    disposers.push(runner(el, triggerEvent, trigger));
  }
  return () => {
    for (const dispose of disposers) dispose();
  };
}
function parseProps(raw) {
  if (!raw) return {};
  try {
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}
export {
  initIslands
};
