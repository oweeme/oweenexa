import type { FormField } from "./validation";

/**
 * Reglas que el HTML no tiene (Fase 81), declaradas con atributos — igual que el resto de `@nexa/forms`:
 *
 * - `data-nexa-match="otroCampo"`: el valor debe ser igual al de ese campo (confirmar contraseña).
 *   Mensaje: `data-nexa-message-match`.
 * - `data-nexa-remote="/api/usuarios/disponible?nombre={value}"`: pregunta a tu API (GET) si el valor sirve.
 *   La API responde JSON `{ "ok": true }` o `{ "ok": false, "message": "Ya existe" }`. Se consulta al salir del
 *   campo (y, con `data-nexa-remote-debounce="400"`, mientras escribes); al enviar, el formulario espera el
 *   resultado. Si la API no responde, NO se bloquea al usuario (el servidor es quien manda al recibir el envío).
 */

const customMessages = new WeakMap<FormField, { match: string; remote: string }>();

function slot(field: FormField) {
    let s = customMessages.get(field);
    if (!s) customMessages.set(field, (s = { match: "", remote: "" }));
    return s;
}

/** Aplica el error propio del campo (`setCustomValidity`): el de coincidencia, si no el remoto. */
function apply(field: FormField): void {
    const s = slot(field);
    field.setCustomValidity(s.match || s.remote);
}

export function applyMatch(form: HTMLFormElement, field: FormField): void {
    const otherName = field.dataset.nexaMatch;
    if (!otherName) return;
    const other = form.elements.namedItem(otherName) as FormField | null;
    const mismatch = !!other && field.value !== other.value && field.value !== "";
    slot(field).match = mismatch ? (field.dataset.nexaMessageMatch ?? "Los valores no coinciden") : "";
    apply(field);
}

/** Los campos que dependen (por `data-nexa-match`) del campo `name`. */
export function dependentsOf(form: HTMLFormElement, field: FormField): FormField[] {
    if (!field.name) return [];
    return Array.from(form.querySelectorAll<FormField>("[data-nexa-match]")).filter((f) => f.dataset.nexaMatch === field.name);
}

export function hasRemote(field: FormField): boolean {
    return !!field.dataset.nexaRemote;
}

interface Checked {
    value: string;
    result: "pending" | "ok" | "bad";
    promise: Promise<boolean>;
    abort: AbortController;
}
const checks = new WeakMap<FormField, Checked>();

/** Consulta a la API si el valor actual sirve. Reutiliza la consulta si el valor no cambió. */
export function checkRemote(field: FormField, onDone: () => void): Promise<boolean> {
    const template = field.dataset.nexaRemote!;
    const value = field.value;
    const previous = checks.get(field);
    if (previous && previous.value === value) return previous.promise;
    previous?.abort.abort();

    // vacío o ya inválido por reglas nativas: nada que preguntarle a la API
    slot(field).remote = "";
    if (value === "" || (!field.validity.valid && !field.validity.customError)) {
        apply(field);
        return Promise.resolve(true);
    }

    const abort = new AbortController();
    const url = template.replace("{value}", encodeURIComponent(value));
    const entry: Checked = { value, result: "pending", abort, promise: Promise.resolve(true) };
    entry.promise = fetch(url, { signal: abort.signal, headers: { Accept: "application/json" }, credentials: "same-origin" })
        .then((r) => (r.ok ? r.json() : null))
        .then((json: { ok?: boolean; message?: string } | null) => {
            if (field.value !== value) return true; // el usuario siguió escribiendo: respuesta vieja
            const ok = !json || json.ok !== false; // sin respuesta útil: no se bloquea
            entry.result = ok ? "ok" : "bad";
            slot(field).remote = ok ? "" : (json?.message ?? field.dataset.nexaMessageRemote ?? "Este valor no está disponible");
            apply(field);
            onDone();
            return ok;
        })
        .catch(() => {
            entry.result = "ok";
            return true;
        });
    checks.set(field, entry);
    return entry.promise;
}

/** ¿Falta preguntar a la API por el valor actual? */
export function needsRemoteCheck(field: FormField): boolean {
    if (!hasRemote(field) || field.value === "") return false;
    const c = checks.get(field);
    return !c || c.value !== field.value || c.result === "pending";
}

export function forgetRemote(field: FormField): void {
    checks.get(field)?.abort.abort();
    checks.delete(field);
    slot(field).remote = "";
    apply(field);
}
