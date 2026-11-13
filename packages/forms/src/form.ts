import { errorTargetFor, getFieldError, getFormFields, type FormField } from "./validation";
import { markDirty, markTouched, setInvalid, CLASS_TOUCHED } from "./state";
import { applyMatch, checkRemote, dependentsOf, forgetRemote, hasRemote, needsRemoteCheck } from "./rules";

/**
 * Activa un `<form data-nexa-form>`: valida con las reglas nativas del
 * HTML (`required`, `type`, `pattern`...), marca `touched`/`dirty`, y
 * muestra el mensaje de error en el `[data-nexa-error-for="campo"]`
 * correspondiente si existe.
 *
 * Si todos los campos son válidos, **no hace nada especial**: deja que
 * el `<form>` siga su envío nativo (o lo que el propio `onSubmit` del
 * desarrollador haga) — es la mitad "mejora progresiva" del contrato: sin
 * este script, el HTML solo (con `required`/`type="email"`) ya valida y
 * envía; con él, se suman mensajes propios y estado visual.
 */
export function initForm(form: HTMLFormElement): () => void {
    const fields = getFormFields(form);
    const cleanups: Array<() => void> = [];

    for (const field of fields) {
        let timer: ReturnType<typeof setTimeout> | undefined;
        const revalidate = () => validateField(form, field);
        const onBlur = () => {
            markTouched(field);
            validateField(form, field);
            if (hasRemote(field)) void checkRemote(field, revalidate);
        };
        const onInput = () => {
            markDirty(field);
            if (hasRemote(field)) {
                forgetRemote(field); // el valor cambió: el resultado anterior ya no vale
                const wait = Number(field.dataset.nexaRemoteDebounce);
                if (wait > 0) {
                    clearTimeout(timer);
                    timer = setTimeout(() => void checkRemote(field, revalidate), wait);
                }
            }
            if (field.classList.contains(CLASS_TOUCHED)) {
                validateField(form, field);
            }
            // un campo «confirmar» depende de este
            for (const dep of dependentsOf(form, field)) {
                if (dep.classList.contains(CLASS_TOUCHED)) validateField(form, dep);
            }
        };
        // Bug real, encontrado con un navegador de verdad (Chromium):
        // si un campo `required` está vacío al enviar, el navegador
        // cancela el envío ANTES de que se dispare `submit` en absoluto
        // (comportamiento estándar de HTML5) — el handler de `onSubmit`
        // de más abajo nunca se llega a ejecutar. Lo único que sí se
        // dispara en ese momento es `invalid` en cada campo inválido, así
        // que hace falta escucharlo aparte para que el estado visual
        // (`nx-invalid`, el mensaje propio) se sincronice igual. Se
        // suprime el globo de validación nativo del navegador porque
        // `data-nexa-error-for` ya es el mecanismo propio para mostrarlo.
        const onInvalid = (event: Event) => {
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

    const onSubmit = (event: Event) => {
        let firstInvalid: FormField | null = null;

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

        // Reglas contra tu API (`data-nexa-remote`): el envío espera la respuesta.
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

function validateField(form: HTMLFormElement, field: FormField): boolean {
    applyMatch(form, field);
    const error = getFieldError(field);
    setInvalid(field, error !== null);

    const target = errorTargetFor(form, field);
    if (target) {
        target.textContent = error ?? "";
    }

    return error === null;
}

/** Activa todos los `[data-nexa-form]` de `root`. */
export function initForms(root: ParentNode = document): () => void {
    const forms = Array.from(root.querySelectorAll<HTMLFormElement>("[data-nexa-form]"));
    const cleanups = forms.map(initForm);
    return () => {
        for (const cleanup of cleanups) cleanup();
    };
}
