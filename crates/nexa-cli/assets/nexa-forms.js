// packages/forms/src/validation.ts
function getFieldError(field) {
  if (field.validity.valid) {
    return null;
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

// packages/forms/src/form.ts
function initForm(form) {
  const fields = getFormFields(form);
  const cleanups = [];
  for (const field of fields) {
    const onBlur = () => {
      markTouched(field);
      validateField(form, field);
    };
    const onInput = () => {
      markDirty(field);
      if (field.classList.contains(CLASS_TOUCHED)) {
        validateField(form, field);
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
    }
  };
  form.addEventListener("submit", onSubmit);
  cleanups.push(() => form.removeEventListener("submit", onSubmit));
  return () => {
    for (const cleanup of cleanups) cleanup();
  };
}
function validateField(form, field) {
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
