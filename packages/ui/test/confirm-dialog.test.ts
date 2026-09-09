import { describe, expect, it } from "vitest";
import { alert, confirm } from "../src";

function activeButton(): HTMLButtonElement {
    return document.activeElement as HTMLButtonElement;
}

describe("ui.confirm()", () => {
    it("resuelve true cuando el usuario clickea Aceptar", async () => {
        document.body.innerHTML = "";
        const pending = confirm("¿Seguro?");

        const dialog = document.querySelector(".nx-dialog") as HTMLElement;
        expect(dialog).not.toBeNull();
        expect(dialog.hasAttribute("hidden")).toBe(false);
        expect(dialog.getAttribute("role")).toBe("alertdialog");

        const confirmButton = Array.from(dialog.querySelectorAll("button")).find((b) => b.textContent === "Aceptar")!;
        confirmButton.click();

        await expect(pending).resolves.toBe(true);
        expect(document.querySelector(".nx-dialog")).toBeNull();
    });

    it("resuelve false cuando el usuario clickea Cancelar", async () => {
        document.body.innerHTML = "";
        const pending = confirm("¿Seguro?");

        const dialog = document.querySelector(".nx-dialog") as HTMLElement;
        const cancelButton = Array.from(dialog.querySelectorAll("button")).find((b) => b.textContent === "Cancelar")!;
        cancelButton.click();

        await expect(pending).resolves.toBe(false);
    });

    it("resuelve false cuando el usuario presiona Escape (cuenta como cancelar)", async () => {
        document.body.innerHTML = "";
        const pending = confirm("¿Seguro?");

        const dialog = document.querySelector(".nx-dialog") as HTMLElement;
        dialog.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

        await expect(pending).resolves.toBe(false);
    });

    it("acepta etiquetas de botón personalizadas", () => {
        document.body.innerHTML = "";
        void confirm("¿Borrar?", { confirmLabel: "Borrar", cancelLabel: "No" });

        const dialog = document.querySelector(".nx-dialog") as HTMLElement;
        const labels = Array.from(dialog.querySelectorAll("button")).map((b) => b.textContent);
        expect(labels).toEqual(["No", "Borrar"]);

        activeButton().click();
    });

    it("apila un confirm() sobre un diálogo ya abierto sin romper ninguno de los dos", async () => {
        document.body.innerHTML = "";
        const dialogA = document.createElement("div");
        dialogA.className = "nx-dialog";
        dialogA.setAttribute("hidden", "");
        const buttonA = document.createElement("button");
        buttonA.textContent = "Borrar (abre confirm)";
        dialogA.appendChild(buttonA);
        document.body.appendChild(dialogA);

        const { openDialog } = await import("../src/dialog");
        openDialog(dialogA);

        const pending = confirm("¿Seguro que querés borrar esto?");
        const confirmDialog = document.querySelectorAll(".nx-dialog")[1] as HTMLElement;
        expect(confirmDialog).not.toBeUndefined();

        const okButton = Array.from(confirmDialog.querySelectorAll("button")).find((b) => b.textContent === "Aceptar")!;
        okButton.click();

        await expect(pending).resolves.toBe(true);
        expect(dialogA.hasAttribute("hidden")).toBe(false);
        expect(document.activeElement).toBe(buttonA);
    });
});

describe("ui.alert()", () => {
    it("resuelve cuando el usuario clickea Aceptar", async () => {
        document.body.innerHTML = "";
        const pending = alert("Guardado con éxito");

        const dialog = document.querySelector(".nx-dialog") as HTMLElement;
        expect(dialog.getAttribute("role")).toBe("alertdialog");

        const okButton = dialog.querySelector("button") as HTMLButtonElement;
        okButton.click();

        await expect(pending).resolves.toBeUndefined();
        expect(document.querySelector(".nx-dialog")).toBeNull();
    });

    it("resuelve cuando el usuario presiona Escape", async () => {
        document.body.innerHTML = "";
        const pending = alert("Guardado con éxito");

        const dialog = document.querySelector(".nx-dialog") as HTMLElement;
        dialog.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));

        await expect(pending).resolves.toBeUndefined();
    });
});
