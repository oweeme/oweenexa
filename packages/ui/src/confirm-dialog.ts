/**
 * `ui.confirm()`/`ui.alert()` (Fase 42, issue #10): el caso común de
 * "confirmar antes de borrar algo" o "avisar que algo se guardó" sin
 * que el desarrollador tenga que escribir el `<div class="nx-dialog">`
 * a mano y cablear `openDialog`/`closeDialog` con callbacks. El HTML se
 * crea en el momento y se descarta al resolver — mismo principio que
 * `ui.notify()` en `toast.ts` (el toast tampoco lo escribe el
 * desarrollador).
 *
 * Ambas devuelven una `Promise` real que resuelve cuando el usuario
 * interactúa: click en un botón, o Escape. Escape en `confirm()` cuenta
 * como cancelar (resuelve `false`), igual que clickear "Cancelar" — el
 * cierre real siempre pasa por `closeDialog()` (vía el manejador de
 * `Escape` que ya tiene `dialog.ts`, o llamado a mano acá desde los
 * botones), y `onDialogClose()` es lo que garantiza que la promesa se
 * resuelva sin importar cuál de esas dos vías cerró el diálogo.
 */

import { closeDialog, onDialogClose, openDialog } from "./dialog";

export interface ConfirmOptions {
    confirmLabel?: string;
    cancelLabel?: string;
}

export interface AlertOptions {
    okLabel?: string;
}

export function confirm(message: string, options: ConfirmOptions = {}): Promise<boolean> {
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

export function alert(message: string, options: AlertOptions = {}): Promise<void> {
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
