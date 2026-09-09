// packages/ui/src/focus-trap.ts
var FOCUSABLE_SELECTOR = 'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';
function getFocusableElements(container2) {
  return Array.from(container2.querySelectorAll(FOCUSABLE_SELECTOR));
}
function trapFocus(container2, event) {
  const focusable = getFocusableElements(container2);
  if (focusable.length === 0) return;
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}

// packages/ui/src/dialog.ts
var previouslyFocused = /* @__PURE__ */ new WeakMap();
var keydownListeners = /* @__PURE__ */ new WeakMap();
var closeCallbacks = /* @__PURE__ */ new WeakMap();
var dialogStack = [];
var BASE_Z_INDEX = 1e3;
var Z_INDEX_STEP = 10;
function openDialog(dialog) {
  previouslyFocused.set(dialog, document.activeElement);
  const below = dialogStack[dialogStack.length - 1];
  below?.setAttribute("aria-hidden", "true");
  dialog.style.zIndex = String(BASE_Z_INDEX + dialogStack.length * Z_INDEX_STEP);
  dialogStack.push(dialog);
  dialog.removeAttribute("hidden");
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  const focusable = getFocusableElements(dialog);
  (focusable[0] ?? dialog).focus();
  const onKeydown = (event) => handleKeydown(dialog, event);
  keydownListeners.set(dialog, onKeydown);
  dialog.addEventListener("keydown", onKeydown);
}
function closeDialog(dialog) {
  const stackIndex = dialogStack.indexOf(dialog);
  if (stackIndex !== -1) {
    dialogStack.splice(stackIndex, 1);
  }
  dialog.style.removeProperty("z-index");
  dialogStack[dialogStack.length - 1]?.removeAttribute("aria-hidden");
  dialog.setAttribute("hidden", "");
  const listener = keydownListeners.get(dialog);
  if (listener) {
    dialog.removeEventListener("keydown", listener);
    keydownListeners.delete(dialog);
  }
  const onClose = closeCallbacks.get(dialog);
  closeCallbacks.delete(dialog);
  const toRestore = previouslyFocused.get(dialog);
  previouslyFocused.delete(dialog);
  toRestore?.focus();
  onClose?.();
}
function onDialogClose(dialog, callback) {
  closeCallbacks.set(dialog, callback);
}
function handleKeydown(dialog, event) {
  if (event.key === "Escape") {
    closeDialog(dialog);
    return;
  }
  if (event.key === "Tab") {
    trapFocus(dialog, event);
  }
}

// packages/ui/src/confirm-dialog.ts
function confirm(message, options = {}) {
  return new Promise((resolve) => {
    let settled = false;
    let outcome = false;
    const dialog = document.createElement("div");
    dialog.className = "nx-dialog";
    dialog.setAttribute("hidden", "");
    const text = document.createElement("p");
    text.textContent = message;
    const cancelButton = document.createElement("button");
    cancelButton.textContent = options.cancelLabel ?? "Cancelar";
    cancelButton.addEventListener("click", () => {
      outcome = false;
      closeDialog(dialog);
    });
    const confirmButton = document.createElement("button");
    confirmButton.textContent = options.confirmLabel ?? "Aceptar";
    confirmButton.addEventListener("click", () => {
      outcome = true;
      closeDialog(dialog);
    });
    dialog.append(text, cancelButton, confirmButton);
    document.body.appendChild(dialog);
    onDialogClose(dialog, () => {
      if (settled) return;
      settled = true;
      dialog.remove();
      resolve(outcome);
    });
    openDialog(dialog);
    dialog.setAttribute("role", "alertdialog");
  });
}
function alert(message, options = {}) {
  return new Promise((resolve) => {
    let settled = false;
    const dialog = document.createElement("div");
    dialog.className = "nx-dialog";
    dialog.setAttribute("hidden", "");
    const text = document.createElement("p");
    text.textContent = message;
    const okButton = document.createElement("button");
    okButton.textContent = options.okLabel ?? "Aceptar";
    okButton.addEventListener("click", () => closeDialog(dialog));
    dialog.append(text, okButton);
    document.body.appendChild(dialog);
    onDialogClose(dialog, () => {
      if (settled) return;
      settled = true;
      dialog.remove();
      resolve();
    });
    openDialog(dialog);
    dialog.setAttribute("role", "alertdialog");
  });
}

// packages/ui/src/drawer.ts
var previouslyFocused2 = /* @__PURE__ */ new WeakMap();
var keydownListeners2 = /* @__PURE__ */ new WeakMap();
function openDrawer(drawer) {
  previouslyFocused2.set(drawer, document.activeElement);
  drawer.removeAttribute("hidden");
  drawer.setAttribute("role", "dialog");
  drawer.setAttribute("aria-modal", "true");
  const focusable = getFocusableElements(drawer);
  (focusable[0] ?? drawer).focus();
  const onKeydown = (event) => handleKeydown2(drawer, event);
  keydownListeners2.set(drawer, onKeydown);
  drawer.addEventListener("keydown", onKeydown);
}
function closeDrawer(drawer) {
  drawer.setAttribute("hidden", "");
  const listener = keydownListeners2.get(drawer);
  if (listener) {
    drawer.removeEventListener("keydown", listener);
    keydownListeners2.delete(drawer);
  }
  const toRestore = previouslyFocused2.get(drawer);
  previouslyFocused2.delete(drawer);
  toRestore?.focus();
}
function handleKeydown2(drawer, event) {
  if (event.key === "Escape") {
    closeDrawer(drawer);
    return;
  }
  if (event.key === "Tab") {
    trapFocus(drawer, event);
  }
}

// packages/ui/src/toast.ts
var DEFAULT_DURATION = 4e3;
var LEAVE_MS = 200;
var container = null;
function notify(options) {
  injectStylesOnce();
  const el = buildToastElement(options);
  getContainer().appendChild(el);
  let closed = false;
  let timer;
  const close = () => {
    if (closed) return;
    closed = true;
    if (timer) clearTimeout(timer);
    el.classList.add("nx-toast-leaving");
    setTimeout(() => el.remove(), LEAVE_MS);
  };
  const duration = options.duration ?? DEFAULT_DURATION;
  if (duration > 0) {
    timer = setTimeout(close, duration);
  }
  el.querySelector(".nx-toast-close")?.addEventListener("click", close);
  return { close };
}
function buildToastElement(options) {
  const el = document.createElement("div");
  el.className = `nx-toast nx-toast-${options.variant ?? "info"}`;
  el.setAttribute("role", "status");
  el.setAttribute("aria-live", "polite");
  const message = document.createElement("span");
  message.className = "nx-toast-message";
  message.textContent = options.message;
  el.appendChild(message);
  const closeBtn = document.createElement("button");
  closeBtn.type = "button";
  closeBtn.className = "nx-toast-close";
  closeBtn.setAttribute("aria-label", "Cerrar");
  closeBtn.textContent = "\xD7";
  el.appendChild(closeBtn);
  return el;
}
function getContainer() {
  if (!container || !container.isConnected) {
    container = document.createElement("div");
    container.className = "nx-toast-container";
    document.body.appendChild(container);
  }
  return container;
}
function injectStylesOnce() {
  if (document.head.querySelector("style[data-nexa-ui-toast]")) return;
  const style = document.createElement("style");
  style.setAttribute("data-nexa-ui-toast", "");
  style.textContent = TOAST_CSS;
  document.head.appendChild(style);
}
var TOAST_CSS = `
.nx-toast-container {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 2147483647;
    max-width: 320px;
}
.nx-toast {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: var(--nx-radius-md, 0.5rem);
    background: var(--nx-color-surface, #ffffff);
    color: var(--nx-color-text, #111827);
    box-shadow: var(--nx-shadow-md, 0 4px 12px rgba(0, 0, 0, 0.15));
    font-family: var(--nx-font-body, system-ui, sans-serif);
    font-size: 0.9rem;
    border-left: 4px solid #6b7280;
    opacity: 1;
    transform: translateY(0);
    transition: opacity ${LEAVE_MS}ms ease, transform ${LEAVE_MS}ms ease;
}
.nx-toast-leaving { opacity: 0; transform: translateY(4px); }
.nx-toast-info { border-left-color: var(--nx-color-primary, #2563eb); }
.nx-toast-success { border-left-color: #16a34a; }
.nx-toast-warning { border-left-color: #d97706; }
.nx-toast-danger { border-left-color: var(--nx-color-danger, #dc2626); }
.nx-toast-message { flex: 1; }
.nx-toast-close {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    color: inherit;
    opacity: 0.6;
    padding: 0;
}
.nx-toast-close:hover { opacity: 1; }
`;

// packages/ui/src/tabs.ts
function selectTab(event) {
  const tab = event.currentTarget;
  const group = tab?.closest(".nx-tab-group");
  const tabId = tab?.dataset.tab;
  if (!group || tabId === void 0) return;
  for (const candidate of group.querySelectorAll(".nx-tab")) {
    const selected = candidate.dataset.tab === tabId;
    candidate.setAttribute("aria-selected", String(selected));
    candidate.tabIndex = selected ? 0 : -1;
  }
  for (const panel of group.querySelectorAll(".nx-tab-panel")) {
    panel.hidden = panel.dataset.tab !== tabId;
  }
}

// packages/ui/src/accordion.ts
function toggleAccordionItem(event) {
  const trigger = event.currentTarget;
  const item = trigger?.closest(".nx-accordion-item");
  const accordion = item?.closest(".nx-accordion");
  const panel = item?.querySelector(".nx-accordion-panel");
  if (!trigger || !item || !panel) return;
  const expanded = trigger.getAttribute("aria-expanded") === "true";
  if (!expanded && accordion?.dataset.exclusive === "true") {
    for (const otherItem of accordion.querySelectorAll(".nx-accordion-item")) {
      if (otherItem === item) continue;
      otherItem.querySelector(".nx-accordion-trigger")?.setAttribute("aria-expanded", "false");
      const otherPanel = otherItem.querySelector(".nx-accordion-panel");
      if (otherPanel) otherPanel.hidden = true;
    }
  }
  trigger.setAttribute("aria-expanded", String(!expanded));
  panel.hidden = expanded;
}

// packages/ui/src/dropdown.ts
var outsideClickListeners = /* @__PURE__ */ new WeakMap();
var keydownListeners3 = /* @__PURE__ */ new WeakMap();
function toggleDropdown(event) {
  const trigger = event.currentTarget;
  const dropdown = trigger?.closest(".nx-dropdown");
  const menu = dropdown?.querySelector(".nx-dropdown-menu");
  if (!trigger || !dropdown || !menu) return;
  if (menu.hasAttribute("hidden")) {
    openDropdownMenu(dropdown, trigger, menu);
  } else {
    closeDropdownMenu(dropdown, trigger, menu);
  }
}
function selectDropdownOption(event) {
  const option = event.currentTarget;
  const dropdown = option?.closest(".nx-dropdown");
  const trigger = dropdown?.querySelector(".nx-dropdown-trigger");
  const menu = dropdown?.querySelector(".nx-dropdown-menu");
  if (!option || !dropdown || !trigger || !menu) return;
  for (const candidate of menu.querySelectorAll(".nx-dropdown-option")) {
    candidate.setAttribute("aria-selected", String(candidate === option));
  }
  trigger.textContent = option.textContent;
  if (option.dataset.value !== void 0) {
    trigger.dataset.value = option.dataset.value;
  }
  closeDropdownMenu(dropdown, trigger, menu);
}
function openDropdownMenu(dropdown, trigger, menu) {
  menu.removeAttribute("hidden");
  trigger.setAttribute("aria-expanded", "true");
  const onOutsideClick = (clickEvent) => {
    if (!dropdown.contains(clickEvent.target)) {
      closeDropdownMenu(dropdown, trigger, menu);
    }
  };
  const onKeydown = (keyEvent) => {
    if (keyEvent.key === "Escape") {
      closeDropdownMenu(dropdown, trigger, menu);
      trigger.focus();
    }
  };
  outsideClickListeners.set(dropdown, onOutsideClick);
  keydownListeners3.set(dropdown, onKeydown);
  document.addEventListener("click", onOutsideClick);
  document.addEventListener("keydown", onKeydown);
}
function closeDropdownMenu(dropdown, trigger, menu) {
  menu.setAttribute("hidden", "");
  trigger.setAttribute("aria-expanded", "false");
  const onOutsideClick = outsideClickListeners.get(dropdown);
  if (onOutsideClick) {
    document.removeEventListener("click", onOutsideClick);
    outsideClickListeners.delete(dropdown);
  }
  const onKeydown = keydownListeners3.get(dropdown);
  if (onKeydown) {
    document.removeEventListener("keydown", onKeydown);
    keydownListeners3.delete(dropdown);
  }
}

// packages/ui/src/table.ts
function sortTable(event) {
  const th = event.currentTarget;
  const table = th?.closest(".nx-table");
  const headerRow = th?.parentElement;
  if (!th || !table || !headerRow) return;
  const columnIndex = Array.from(headerRow.children).indexOf(th);
  if (columnIndex === -1) return;
  const nextDir = th.getAttribute("aria-sort") === "ascending" ? "descending" : "ascending";
  for (const header of Array.from(headerRow.children)) {
    header.removeAttribute("aria-sort");
  }
  th.setAttribute("aria-sort", nextDir);
  const tbody = table.querySelector("tbody");
  if (!tbody) return;
  const direction = nextDir === "ascending" ? 1 : -1;
  const rows = Array.from(tbody.querySelectorAll("tr"));
  rows.sort((rowA, rowB) => {
    const cellA = rowA.children[columnIndex]?.textContent?.trim() ?? "";
    const cellB = rowB.children[columnIndex]?.textContent?.trim() ?? "";
    const numA = Number(cellA);
    const numB = Number(cellB);
    if (cellA !== "" && cellB !== "" && !Number.isNaN(numA) && !Number.isNaN(numB)) {
      return (numA - numB) * direction;
    }
    return cellA.localeCompare(cellB) * direction;
  });
  for (const row of rows) {
    tbody.appendChild(row);
  }
}
function filterTable(table, query) {
  const tbody = table.querySelector("tbody");
  if (!tbody) return;
  const q = query.trim().toLowerCase();
  for (const row of Array.from(tbody.querySelectorAll("tr"))) {
    const text = (row.textContent ?? "").toLowerCase();
    row.hidden = q.length > 0 && !text.includes(q);
  }
}

// packages/reactivity/src/reactive.ts
var activeEffect = null;
var pending = /* @__PURE__ */ new Set();
var flushScheduled = false;
function schedule(effect2) {
  pending.add(effect2);
  if (flushScheduled) return;
  flushScheduled = true;
  queueMicrotask(flush);
}
function flush() {
  flushScheduled = false;
  const effects = Array.from(pending);
  pending.clear();
  for (const effect2 of effects) {
    effect2.execute();
  }
}
var Signal = class {
  #value;
  #subscribers = /* @__PURE__ */ new Set();
  constructor(initial) {
    this.#value = initial;
  }
  get value() {
    if (activeEffect) {
      this.#subscribers.add(activeEffect);
      activeEffect.deps.add(this);
    }
    return this.#value;
  }
  set value(next) {
    if (Object.is(next, this.#value)) return;
    this.#value = next;
    for (const effect2 of this.#subscribers) {
      schedule(effect2);
    }
  }
  /** Lee el valor sin suscribirse: no crea una dependencia. */
  peek() {
    return this.#value;
  }
  /** Usado por los efectos al limpiar sus dependencias antes de re-ejecutarse. */
  unsubscribe(effect2) {
    this.#subscribers.delete(effect2);
  }
};
function state(initial) {
  return new Signal(initial);
}
var EffectImpl = class {
  deps = /* @__PURE__ */ new Set();
  #fn;
  #disposed = false;
  constructor(fn) {
    this.#fn = fn;
    this.execute();
  }
  execute() {
    if (this.#disposed) return;
    this.#cleanup();
    const previous = activeEffect;
    activeEffect = this;
    try {
      this.#fn();
    } finally {
      activeEffect = previous;
    }
  }
  #cleanup() {
    for (const dep of this.deps) {
      dep.unsubscribe(this);
    }
    this.deps.clear();
  }
  dispose() {
    this.#disposed = true;
    this.#cleanup();
    pending.delete(this);
  }
};
function createEffect(fn) {
  const instance = new EffectImpl(fn);
  return () => instance.dispose();
}
var effect = Object.assign(createEffect, { client: createEffect });

// packages/ui/src/search.ts
function createPageSearch(container2, itemSelector = "[data-searchable]") {
  const query = state("");
  effect(() => {
    const q = query.value.trim().toLowerCase();
    for (const item of Array.from(container2.querySelectorAll(itemSelector))) {
      const text = (item.textContent ?? "").toLowerCase();
      item.hidden = q.length > 0 && !text.includes(q);
    }
  });
  return {
    setQuery(value) {
      query.value = value;
    }
  };
}

// packages/ui/src/index.ts
var ui = {
  openDialog,
  closeDialog,
  confirm,
  alert,
  openDrawer,
  closeDrawer,
  notify,
  selectTab,
  toggleAccordionItem,
  toggleDropdown,
  selectDropdownOption,
  sortTable,
  filterTable,
  createPageSearch
};
export {
  alert,
  closeDialog,
  closeDrawer,
  confirm,
  createPageSearch,
  filterTable,
  notify,
  openDialog,
  openDrawer,
  selectDropdownOption,
  selectTab,
  sortTable,
  toggleAccordionItem,
  toggleDropdown,
  ui
};
