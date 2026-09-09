import { describe, expect, it } from "vitest";
import { closeDialog, openDialog } from "../src";

function setupDialog(): { trigger: HTMLButtonElement; dialog: HTMLDivElement; first: HTMLButtonElement; last: HTMLButtonElement } {
    document.body.innerHTML = "";

    const trigger = document.createElement("button");
    trigger.textContent = "Abrir";
    document.body.appendChild(trigger);

    const dialog = document.createElement("div");
    dialog.className = "nx-dialog";
    dialog.setAttribute("hidden", "");
    document.body.appendChild(dialog);

    const first = document.createElement("button");
    first.textContent = "Primero";
    const last = document.createElement("button");
    last.textContent = "Último";
    dialog.append(first, last);

    trigger.focus();

    return { trigger, dialog, first, last };
}

describe("openDialog / closeDialog", () => {
    it("quita `hidden` y pone los atributos ARIA de un diálogo modal", () => {
        const { dialog } = setupDialog();
        openDialog(dialog);

        expect(dialog.hasAttribute("hidden")).toBe(false);
        expect(dialog.getAttribute("role")).toBe("dialog");
        expect(dialog.getAttribute("aria-modal")).toBe("true");
    });

    it("mueve el foco dentro del diálogo al abrir", () => {
        const { dialog, first } = setupDialog();
        openDialog(dialog);

        expect(document.activeElement).toBe(first);
    });

    it("devuelve el foco a quien abrió el diálogo, al cerrarlo", () => {
        const { trigger, dialog } = setupDialog();
        openDialog(dialog);
        closeDialog(dialog);

        expect(dialog.hasAttribute("hidden")).toBe(true);
        expect(document.activeElement).toBe(trigger);
    });

    it("Escape cierra el diálogo", () => {
        const { dialog } = setupDialog();
        openDialog(dialog);

        dialog.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

        expect(dialog.hasAttribute("hidden")).toBe(true);
    });

    it("Tab en el último elemento enfocable vuelve al primero (focus trap)", () => {
        const { dialog, first, last } = setupDialog();
        openDialog(dialog);
        last.focus();

        const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
        dialog.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(first);
    });

    it("Shift+Tab en el primer elemento enfocable va al último (focus trap)", () => {
        const { dialog, first, last } = setupDialog();
        openDialog(dialog);
        first.focus();

        const event = new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true, cancelable: true });
        dialog.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(last);
    });
});

function setupNestedDialog(opener: HTMLElement): { dialog: HTMLDivElement; first: HTMLButtonElement; last: HTMLButtonElement } {
    const dialog = document.createElement("div");
    dialog.className = "nx-dialog";
    dialog.setAttribute("hidden", "");
    document.body.appendChild(dialog);

    const first = document.createElement("button");
    first.textContent = "B-primero";
    const last = document.createElement("button");
    last.textContent = "B-último";
    dialog.append(first, last);

    opener.focus();
    return { dialog, first, last };
}

describe("apilamiento de diálogos (Fase 42)", () => {
    it("abrir un segundo diálogo no rompe el focus trap del primero: Tab en B no escapa a A", () => {
        const { dialog: dialogA, last: lastA } = setupDialog();
        openDialog(dialogA);

        const { dialog: dialogB, first: firstB, last: lastB } = setupNestedDialog(lastA);
        openDialog(dialogB);

        expect(document.activeElement).toBe(firstB);

        lastB.focus();
        const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
        dialogB.dispatchEvent(event);

        expect(event.defaultPrevented).toBe(true);
        expect(document.activeElement).toBe(firstB);
    });

    it("cerrar el diálogo superior devuelve el foco al elemento del diálogo inferior que lo abrió, no a document.body", () => {
        const { dialog: dialogA, last: lastA } = setupDialog();
        openDialog(dialogA);

        const { dialog: dialogB } = setupNestedDialog(lastA);
        openDialog(dialogB);

        closeDialog(dialogB);

        expect(dialogB.hasAttribute("hidden")).toBe(true);
        expect(dialogA.hasAttribute("hidden")).toBe(false);
        expect(document.activeElement).toBe(lastA);
    });

    it("el diálogo de abajo queda con aria-hidden mientras el de arriba está abierto, y se restaura al cerrarlo", () => {
        const { dialog: dialogA, last: lastA } = setupDialog();
        openDialog(dialogA);
        expect(dialogA.hasAttribute("aria-hidden")).toBe(false);

        const { dialog: dialogB } = setupNestedDialog(lastA);
        openDialog(dialogB);
        expect(dialogA.getAttribute("aria-hidden")).toBe("true");

        closeDialog(dialogB);
        expect(dialogA.hasAttribute("aria-hidden")).toBe(false);
    });

    it("el diálogo apilado encima tiene mayor z-index que el de abajo", () => {
        const { dialog: dialogA, last: lastA } = setupDialog();
        openDialog(dialogA);

        const { dialog: dialogB } = setupNestedDialog(lastA);
        openDialog(dialogB);

        expect(Number(dialogB.style.zIndex)).toBeGreaterThan(Number(dialogA.style.zIndex));
    });
});
