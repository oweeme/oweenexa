import type { FormField } from "./validation";

/**
 * Clases de estado que `@nexa/ui` (o el CSS del propio proyecto) puede
 * usar para dar estilo — Nexa no impone ningún look, solo pone y quita
 * las clases en el momento correcto.
 */
export const CLASS_TOUCHED = "nx-touched";
export const CLASS_DIRTY = "nx-dirty";
export const CLASS_INVALID = "nx-invalid";

export function markTouched(field: FormField): void {
    field.classList.add(CLASS_TOUCHED);
}

export function markDirty(field: FormField): void {
    field.classList.add(CLASS_DIRTY);
}

export function setInvalid(field: FormField, isInvalid: boolean): void {
    field.classList.toggle(CLASS_INVALID, isInvalid);
    field.setAttribute("aria-invalid", isInvalid ? "true" : "false");
}
