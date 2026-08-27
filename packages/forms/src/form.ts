import { errorTargetFor, getFieldError, getFormFields, type FormField } from "./validation";
import { markDirty, markTouched, setInvalid, CLASS_TOUCHED } from "./state";

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
        }
    };

    form.addEventListener("submit", onSubmit);
    cleanups.push(() => form.removeEventListener("submit", onSubmit));

    return () => {
        for (const cleanup of cleanups) cleanup();
    };
}

function validateField(form: HTMLFormElement, field: FormField): boolean {
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
