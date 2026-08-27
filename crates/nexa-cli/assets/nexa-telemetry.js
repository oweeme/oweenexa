// packages/telemetry/src/errors.ts
function observeErrors(onError, options = {}) {
  const win = options.window ?? (typeof window !== "undefined" ? window : void 0);
  if (!win) {
    return () => {
    };
  }
  const handleError = (event) => {
    const location = event.filename ? ` (${event.filename}:${event.lineno}:${event.colno})` : "";
    onError({ name: "error", message: `${event.message}${location}` });
  };
  const handleRejection = (event) => {
    const reason = event.reason instanceof Error ? event.reason.message : String(event.reason);
    onError({ name: "unhandledrejection", message: reason });
  };
  win.addEventListener("error", handleError);
  win.addEventListener("unhandledrejection", handleRejection);
  return () => {
    win.removeEventListener("error", handleError);
    win.removeEventListener("unhandledrejection", handleRejection);
  };
}

// packages/telemetry/src/report.ts
var defaultReportSender = (endpoint, report) => {
  const body = JSON.stringify(report);
  if (typeof navigator !== "undefined" && typeof navigator.sendBeacon === "function") {
    navigator.sendBeacon(endpoint, body);
    return;
  }
  if (typeof fetch === "function") {
    void fetch(endpoint, { method: "POST", body, keepalive: true }).catch(() => {
    });
  }
};

// packages/telemetry/src/vitals.ts
function observeVitals(onVital, options = {}) {
  const Ctor = options.PerformanceObserverCtor ?? (typeof PerformanceObserver !== "undefined" ? PerformanceObserver : void 0);
  if (!Ctor) {
    return () => {
    };
  }
  const observers = [];
  const supported = Ctor.supportedEntryTypes ?? [];
  let lastLcp = 0;
  observeType(Ctor, supported, "largest-contentful-paint", observers, (entries) => {
    const last = entries[entries.length - 1];
    if (!last) return;
    lastLcp = last.renderTime || last.loadTime || last.startTime;
    onVital({ name: "LCP", value: lastLcp });
  });
  let cls = 0;
  observeType(Ctor, supported, "layout-shift", observers, (entries) => {
    for (const entry of entries) {
      if (!entry.hadRecentInput) {
        cls += entry.value;
      }
    }
    onVital({ name: "CLS", value: cls });
  });
  observeType(Ctor, supported, "first-input", observers, (entries) => {
    const first = entries[0];
    if (!first) return;
    onVital({ name: "INP", value: first.processingStart - first.startTime });
  });
  return () => {
    for (const observer of observers) observer.disconnect();
  };
}
function observeType(Ctor, supported, type, observers, onEntries) {
  if (supported.length > 0 && !supported.includes(type)) return;
  try {
    const observer = new Ctor((list) => onEntries(list.getEntries()));
    observer.observe({ type, buffered: true });
    observers.push(observer);
  } catch {
  }
}

// packages/telemetry/src/client.ts
function initTelemetry(options) {
  const sendReport = options.sendReport ?? defaultReportSender;
  const doc = options.document ?? document;
  const path = doc.defaultView?.location.pathname ?? "/";
  const emit = (report) => {
    options.onReport?.(report);
    sendReport(options.endpoint, report);
  };
  const stopVitals = observeVitals((vital) => {
    emit({ kind: "vital", name: vital.name, value: vital.value, path, timestamp: Date.now() });
  });
  const stopErrors = observeErrors(
    (error) => {
      emit({ kind: "error", name: error.name, detail: error.message, path, timestamp: Date.now() });
    },
    { window: options.window }
  );
  return () => {
    stopVitals();
    stopErrors();
  };
}
export {
  defaultReportSender,
  initTelemetry,
  observeErrors,
  observeVitals
};
