// packages/ui/src/dialog.ts
var previouslyFocused = /* @__PURE__ */ new WeakMap();
var keydownListeners = /* @__PURE__ */ new WeakMap();
var FOCUSABLE_SELECTOR = 'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])';
function openDialog(dialog) {
  previouslyFocused.set(dialog, document.activeElement);
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
  dialog.setAttribute("hidden", "");
  const listener = keydownListeners.get(dialog);
  if (listener) {
    dialog.removeEventListener("keydown", listener);
    keydownListeners.delete(dialog);
  }
  const toRestore = previouslyFocused.get(dialog);
  previouslyFocused.delete(dialog);
  toRestore?.focus();
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
function trapFocus(dialog, event) {
  const focusable = getFocusableElements(dialog);
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
function getFocusableElements(container) {
  return Array.from(container.querySelectorAll(FOCUSABLE_SELECTOR));
}

// packages/ui/src/index.ts
var ui = { openDialog, closeDialog };
export {
  closeDialog,
  openDialog,
  ui
};
