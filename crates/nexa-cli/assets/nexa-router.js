// packages/router/src/fetcher.ts
var defaultFetcher = (path) => fetch(path).then((res) => res.text());

// packages/router/src/navigate.ts
var SLOT_SELECTOR = "[data-nexa-slot]";
var MANIFEST_SELECTOR = "script[data-nexa-manifest]";
function initRouter(options = {}) {
  const root = options.root ?? document;
  const fetchPage = options.fetchPage ?? defaultFetcher;
  const cache = options.cache ?? /* @__PURE__ */ new Map();
  const onNavigate = options.onNavigate;
  const onClick = (event) => {
    if (event.defaultPrevented || event.button !== 0) return;
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    const target = event.target;
    const anchor = target?.closest?.("a[href]");
    if (!anchor || !isInternalNavigableLink(anchor)) return;
    event.preventDefault();
    void navigate(anchor.href, true);
  };
  const onPopState = () => {
    void navigate(location.href, false);
  };
  async function navigate(url, push) {
    const path = new URL(url, location.href).pathname;
    const html = cache.get(path) ?? await fetchPage(path);
    cache.delete(path);
    const reactivationRoot = applyPage(root, html);
    onNavigate?.(reactivationRoot);
    if (push) {
      history.pushState({}, "", url);
    }
  }
  root.addEventListener("click", onClick);
  window.addEventListener("popstate", onPopState);
  return () => {
    root.removeEventListener("click", onClick);
    window.removeEventListener("popstate", onPopState);
  };
}
function isInternalNavigableLink(anchor) {
  if (anchor.target && anchor.target !== "_self") return false;
  if (anchor.hasAttribute("download")) return false;
  if (anchor.hasAttribute("data-nexa-reload")) return false;
  const url = new URL(anchor.href, location.href);
  return url.origin === location.origin;
}
function applyPage(root, html) {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  root.title = parsed.title;
  const currentSlot = root.querySelector(SLOT_SELECTOR);
  const incomingSlot = parsed.querySelector(SLOT_SELECTOR);
  if (currentSlot && incomingSlot) {
    currentSlot.innerHTML = incomingSlot.innerHTML;
    syncManifestScript(root, parsed);
    return currentSlot;
  }
  root.body.innerHTML = parsed.body.innerHTML;
  return root;
}
function syncManifestScript(root, parsed) {
  const current = root.querySelector(MANIFEST_SELECTOR);
  const incoming = parsed.querySelector(MANIFEST_SELECTOR);
  if (incoming) {
    if (current) {
      current.textContent = incoming.textContent;
    } else {
      root.body.appendChild(incoming.cloneNode(true));
    }
  } else {
    current?.remove();
  }
}

// packages/router/src/prefetch.ts
function initPrefetch(options = {}) {
  const root = options.root ?? document;
  const fetchPage = options.fetchPage ?? defaultFetcher;
  const cache = options.cache ?? /* @__PURE__ */ new Map();
  const hoverDelay = options.hoverDelay ?? 60;
  const timers = /* @__PURE__ */ new WeakMap();
  function schedule(anchor) {
    if (timers.has(anchor)) return;
    const timer = setTimeout(() => {
      timers.delete(anchor);
      void prefetch(anchor);
    }, hoverDelay);
    timers.set(anchor, timer);
  }
  function cancel(anchor) {
    const timer = timers.get(anchor);
    if (timer !== void 0) {
      clearTimeout(timer);
      timers.delete(anchor);
    }
  }
  async function prefetch(anchor) {
    const url = new URL(anchor.href, location.href);
    if (url.origin !== location.origin || cache.has(url.pathname)) return;
    try {
      cache.set(url.pathname, await fetchPage(url.pathname));
    } catch {
    }
  }
  const anchorFrom = (event) => event.target?.closest?.("a[href]");
  const onPointerOver = (event) => {
    const anchor = anchorFrom(event);
    if (anchor) schedule(anchor);
  };
  const onPointerOut = (event) => {
    const anchor = anchorFrom(event);
    if (anchor) cancel(anchor);
  };
  const onTouchStart = (event) => {
    const anchor = anchorFrom(event);
    if (anchor) void prefetch(anchor);
  };
  root.addEventListener("pointerover", onPointerOver);
  root.addEventListener("pointerout", onPointerOut);
  root.addEventListener("touchstart", onTouchStart, { passive: true });
  return () => {
    root.removeEventListener("pointerover", onPointerOver);
    root.removeEventListener("pointerout", onPointerOut);
    root.removeEventListener("touchstart", onTouchStart);
  };
}
export {
  initPrefetch,
  initRouter
};
