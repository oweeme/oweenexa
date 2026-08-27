// packages/dev-client/src/swap.ts
var defaultPageFetcher = (path) => fetch(path, { cache: "no-store" }).then((res) => res.text());
function swapDocument(doc, html) {
  const parsed = new DOMParser().parseFromString(html, "text/html");
  const scripts = Array.from(parsed.body.querySelectorAll("script"));
  for (const script of scripts) {
    script.remove();
  }
  const scrollX = doc.defaultView?.scrollX ?? 0;
  const scrollY = doc.defaultView?.scrollY ?? 0;
  doc.title = parsed.title;
  doc.body.innerHTML = parsed.body.innerHTML;
  for (const script of scripts) {
    const fresh = doc.createElement("script");
    for (const attr of Array.from(script.attributes)) {
      fresh.setAttribute(attr.name, attr.value);
    }
    fresh.textContent = script.textContent;
    doc.body.appendChild(fresh);
  }
  doc.defaultView?.scrollTo(scrollX, scrollY);
}

// packages/dev-client/src/version.ts
var defaultVersionFetcher = () => fetch("/__nexa_dev__/version", { cache: "no-store" }).then((res) => res.text());

// packages/dev-client/src/poller.ts
function createPoller(options = {}) {
  const fetchVersion = options.fetchVersion ?? defaultVersionFetcher;
  const fetchPage = options.fetchPage ?? defaultPageFetcher;
  const doc = options.document ?? document;
  let baseline = null;
  async function tick() {
    try {
      const version = await fetchVersion();
      if (baseline === null) {
        baseline = version;
        return;
      }
      if (version === baseline) {
        return;
      }
      baseline = version;
      const path = doc.defaultView?.location.pathname ?? "/";
      const html = await fetchPage(path);
      swapDocument(doc, html);
    } catch {
    }
  }
  return { tick };
}

// packages/dev-client/src/client.ts
function initDevClient(options = {}) {
  const intervalMs = options.intervalMs ?? 400;
  const poller = createPoller(options);
  void poller.tick();
  const id = setInterval(() => void poller.tick(), intervalMs);
  return () => clearInterval(id);
}
export {
  createPoller,
  defaultPageFetcher,
  defaultVersionFetcher,
  initDevClient,
  swapDocument
};
