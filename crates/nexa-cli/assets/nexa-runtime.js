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
function activateManually(el) {
  const trigger = manualTriggers.get(el);
  return trigger ? trigger() : Promise.resolve();
}
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

// packages/runtime/src/activate.ts
var defaultLoader = (path) => import(
  /* @vite-ignore */
  path
);
var runners = { interaction, visible, idle, load, manual };
function initActivation(manifest, options = {}) {
  const root = options.root ?? document;
  const loadModule = options.loadModule ?? defaultLoader;
  const disposers = [];
  for (const [id, entry] of Object.entries(manifest)) {
    const elements = root.querySelectorAll(`[data-nexa="${id}"]`);
    if (elements.length === 0) continue;
    for (const el of Array.from(elements)) {
      const trigger = async () => {
        const mod = await loadModule(entry.module);
        mod.default(el);
      };
      const runner = runners[entry.strategy] ?? interaction;
      disposers.push(runner(el, entry.event, trigger));
    }
  }
  return () => {
    for (const dispose of disposers) dispose();
  };
}
export {
  activateManually,
  idle,
  initActivation,
  interaction,
  load,
  manual,
  visible
};
