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

// packages/ui/src/index.ts
var ui = { openDialog, closeDialog, confirm, alert, openDrawer, closeDrawer, notify };
export {
  alert,
  closeDialog,
  closeDrawer,
  confirm,
  notify,
  openDialog,
  openDrawer,
  ui
};
