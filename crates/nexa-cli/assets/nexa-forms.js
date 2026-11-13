// packages/forms/src/validation.ts
function getFieldError(field) {
  if (field.validity.valid) {
    return null;
  }
  if (field.validity.customError) {
    return field.validationMessage;
  }
  return field.dataset.nexaMessage ?? field.validationMessage;
}
function errorTargetFor(form, field) {
  if (!field.name) return null;
  return form.querySelector(`[data-nexa-error-for="${field.name}"]`);
}
function getFormFields(form) {
  return Array.from(form.elements).filter(
    (el) => el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement
  );
}

// packages/forms/src/state.ts
var CLASS_TOUCHED = "nx-touched";
var CLASS_DIRTY = "nx-dirty";
var CLASS_INVALID = "nx-invalid";
function markTouched(field) {
  field.classList.add(CLASS_TOUCHED);
}
function markDirty(field) {
  field.classList.add(CLASS_DIRTY);
}
function setInvalid(field, isInvalid) {
  field.classList.toggle(CLASS_INVALID, isInvalid);
  field.setAttribute("aria-invalid", isInvalid ? "true" : "false");
}

// packages/forms/src/rules.ts
var customMessages = /* @__PURE__ */ new WeakMap();
function slot(field) {
  let s = customMessages.get(field);
  if (!s) customMessages.set(field, s = { match: "", remote: "" });
  return s;
}
function apply(field) {
  const s = slot(field);
  field.setCustomValidity(s.match || s.remote);
}
function applyMatch(form, field) {
  const otherName = field.dataset.nexaMatch;
  if (!otherName) return;
  const other = form.elements.namedItem(otherName);
  const mismatch = !!other && field.value !== other.value && field.value !== "";
  slot(field).match = mismatch ? field.dataset.nexaMessageMatch ?? "Los valores no coinciden" : "";
  apply(field);
}
function dependentsOf(form, field) {
  if (!field.name) return [];
  return Array.from(form.querySelectorAll("[data-nexa-match]")).filter((f) => f.dataset.nexaMatch === field.name);
}
function hasRemote(field) {
  return !!field.dataset.nexaRemote;
}
var checks = /* @__PURE__ */ new WeakMap();
function checkRemote(field, onDone) {
  const template = field.dataset.nexaRemote;
  const value = field.value;
  const previous = checks.get(field);
  if (previous && previous.value === value) return previous.promise;
  previous?.abort.abort();
  slot(field).remote = "";
  if (value === "" || !field.validity.valid && !field.validity.customError) {
    apply(field);
    return Promise.resolve(true);
  }
  const abort = new AbortController();
  const url = template.replace("{value}", encodeURIComponent(value));
  const entry = { value, result: "pending", abort, promise: Promise.resolve(true) };
  entry.promise = fetch(url, { signal: abort.signal, headers: { Accept: "application/json" }, credentials: "same-origin" }).then((r) => r.ok ? r.json() : null).then((json) => {
    if (field.value !== value) return true;
    const ok = !json || json.ok !== false;
    entry.result = ok ? "ok" : "bad";
    slot(field).remote = ok ? "" : json?.message ?? field.dataset.nexaMessageRemote ?? "Este valor no est\xE1 disponible";
    apply(field);
    onDone();
    return ok;
  }).catch(() => {
    entry.result = "ok";
    return true;
  });
  checks.set(field, entry);
  return entry.promise;
}
function needsRemoteCheck(field) {
  if (!hasRemote(field) || field.value === "") return false;
  const c = checks.get(field);
  return !c || c.value !== field.value || c.result === "pending";
}
function forgetRemote(field) {
  checks.get(field)?.abort.abort();
  checks.delete(field);
  slot(field).remote = "";
  apply(field);
}

// packages/forms/src/form.ts
function initForm(form) {
  const fields = getFormFields(form);
  const cleanups = [];
  for (const field of fields) {
    let timer;
    const revalidate = () => validateField(form, field);
    const onBlur = () => {
      markTouched(field);
      validateField(form, field);
      if (hasRemote(field)) void checkRemote(field, revalidate);
    };
    const onInput = () => {
      markDirty(field);
      if (hasRemote(field)) {
        forgetRemote(field);
        const wait = Number(field.dataset.nexaRemoteDebounce);
        if (wait > 0) {
          clearTimeout(timer);
          timer = setTimeout(() => void checkRemote(field, revalidate), wait);
        }
      }
      if (field.classList.contains(CLASS_TOUCHED)) {
        validateField(form, field);
      }
      for (const dep of dependentsOf(form, field)) {
        if (dep.classList.contains(CLASS_TOUCHED)) validateField(form, dep);
      }
    };
    const onInvalid = (event) => {
      event.preventDefault();
      markTouched(field);
      validateField(form, field);
    };
    field.addEventListener("blur", onBlur);
    field.addEventListener("input", onInput);
    field.addEventListener("invalid", onInvalid);
    cleanups.push(() => {
      clearTimeout(timer);
      field.removeEventListener("blur", onBlur);
      field.removeEventListener("input", onInput);
      field.removeEventListener("invalid", onInvalid);
    });
  }
  const onSubmit = (event) => {
    let firstInvalid = null;
    for (const field of fields) {
      markTouched(field);
      if (!validateField(form, field) && !firstInvalid) {
        firstInvalid = field;
      }
    }
    if (firstInvalid) {
      event.preventDefault();
      firstInvalid.focus();
      return;
    }
    const pending = fields.filter(needsRemoteCheck);
    if (pending.length > 0) {
      event.preventDefault();
      void Promise.all(pending.map((f) => checkRemote(f, () => validateField(form, f)))).then(() => {
        const bad = pending.find((f) => !f.validity.valid);
        if (bad) {
          validateField(form, bad);
          bad.focus();
        } else if (typeof form.requestSubmit === "function") {
          form.requestSubmit();
        } else {
          form.submit();
        }
      });
    }
  };
  form.addEventListener("submit", onSubmit);
  cleanups.push(() => form.removeEventListener("submit", onSubmit));
  return () => {
    for (const cleanup of cleanups) cleanup();
  };
}
function validateField(form, field) {
  applyMatch(form, field);
  const error = getFieldError(field);
  setInvalid(field, error !== null);
  const target = errorTargetFor(form, field);
  if (target) {
    target.textContent = error ?? "";
  }
  return error === null;
}
function initForms(root = document) {
  const forms = Array.from(root.querySelectorAll("[data-nexa-form]"));
  const cleanups = forms.map(initForm);
  return () => {
    for (const cleanup of cleanups) cleanup();
  };
}
export {
  CLASS_DIRTY,
  CLASS_INVALID,
  CLASS_TOUCHED,
  errorTargetFor,
  getFieldError,
  getFormFields,
  initForm,
  initForms,
  markDirty,
  markTouched,
  setInvalid
};
