/**
 * Se apoya en la validación nativa del navegador (`required`,
 * `type="email"`, `pattern`, `minlength`...) en vez de reinventar reglas:
 * eso es lo que hace que un formulario de Nexa funcione *sin JS* — el
 * HTML por sí solo ya valida. Esta capa solo añade mensajes propios y
 * los muestra donde el desarrollador diga.
 */

export type FormField = HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement;

/**
 * `null` si el campo es válido. Si no, el mensaje a mostrar:
 * `data-nexa-message` si el desarrollador puso uno, si no el
 * `validationMessage` nativo del navegador.
 *
 * Lee `field.validity`/`field.validationMessage` directamente — **no**
 * llama a `field.checkValidity()` (que siempre está actualizado sin
 * necesidad de forzar nada, per spec: es una `ValidityState` viva). Es
 * deliberado: `checkValidity()` dispara su propio evento `invalid` si el
 * campo no es válido, y esta función se llama justamente *desde* el
 * handler de `invalid` (Fase 15, ver `form.ts`) — volver a llamarla ahí
 * volvería a disparar `invalid`, en un ciclo infinito. Se encontró como
 * un cuelgue real de la suite de tests (recursión infinita en
 * happy-dom) al escribir la prueba de extremo a extremo que confirmó el
 * fix original en un navegador real.
 */
export function getFieldError(field: FormField): string | null {
    if (field.validity.valid) {
        return null;
    }
    return field.dataset.nexaMessage ?? field.validationMessage;
}

export function errorTargetFor(form: HTMLFormElement, field: FormField): HTMLElement | null {
    if (!field.name) return null;
    return form.querySelector<HTMLElement>(`[data-nexa-error-for="${field.name}"]`);
}

export function getFormFields(form: HTMLFormElement): FormField[] {
    return Array.from(form.elements).filter(
        (el): el is FormField =>
            el instanceof HTMLInputElement || el instanceof HTMLTextAreaElement || el instanceof HTMLSelectElement,
    );
}
