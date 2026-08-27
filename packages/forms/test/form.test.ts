import { describe, expect, it } from "vitest";
import { initForm } from "../src";

function buildContactForm(): { form: HTMLFormElement; email: HTMLInputElement; errorEl: HTMLElement } {
    document.body.innerHTML = `
        <form data-nexa-form novalidate>
            <input name="email" type="email" required />
            <span data-nexa-error-for="email"></span>
            <button type="submit">Enviar</button>
        </form>
    `;
    const form = document.querySelector("form") as HTMLFormElement;
    const email = form.elements.namedItem("email") as HTMLInputElement;
    const errorEl = form.querySelector('[data-nexa-error-for="email"]') as HTMLElement;
    return { form, email, errorEl };
}

function submit(form: HTMLFormElement): Event {
    const event = new Event("submit", { bubbles: true, cancelable: true });
    form.dispatchEvent(event);
    return event;
}

describe("initForm — mejora progresiva", () => {
    it("bloquea el envío y marca el campo inválido si un campo requerido está vacío", () => {
        const { form, email } = buildContactForm();
        initForm(form);

        const event = submit(form);

        expect(event.defaultPrevented).toBe(true);
        expect(email.classList.contains("nx-invalid")).toBe(true);
        expect(email.getAttribute("aria-invalid")).toBe("true");
        // El texto exacto del mensaje nativo lo decide el navegador (varía
        // por idioma/motor); `data-nexa-message` es la forma soportada de
        // controlarlo — probado aparte, más abajo.
    });

    it("no bloquea el envío si el formulario es válido", () => {
        const { form, email } = buildContactForm();
        initForm(form);
        email.value = "persona@example.com";

        const event = submit(form);

        expect(event.defaultPrevented).toBe(false);
        expect(email.classList.contains("nx-invalid")).toBe(false);
    });

    it("usa data-nexa-message en vez del mensaje nativo, si está presente", () => {
        const { form, email, errorEl } = buildContactForm();
        email.dataset.nexaMessage = "Escribe un correo válido";
        initForm(form);

        submit(form);

        expect(errorEl.textContent).toBe("Escribe un correo válido");
    });

    it("marca `touched` al perder el foco, y `dirty` al escribir", () => {
        const { form, email } = buildContactForm();
        initForm(form);

        email.dispatchEvent(new Event("input", { bubbles: true }));
        expect(email.classList.contains("nx-dirty")).toBe(true);
        expect(email.classList.contains("nx-touched")).toBe(false);

        email.dispatchEvent(new Event("blur", { bubbles: true }));
        expect(email.classList.contains("nx-touched")).toBe(true);
    });

    it("revalida en caliente solo después de haber sido tocado", () => {
        const { form, email } = buildContactForm();
        initForm(form);

        // Todavía no se tocó: escribir un valor inválido no debe marcar error aún.
        email.value = "no-es-un-email";
        email.dispatchEvent(new Event("input", { bubbles: true }));
        expect(email.classList.contains("nx-invalid")).toBe(false);

        email.dispatchEvent(new Event("blur", { bubbles: true }));
        expect(email.classList.contains("nx-invalid")).toBe(true);

        email.value = "ahora@si.com";
        email.dispatchEvent(new Event("input", { bubbles: true }));
        expect(email.classList.contains("nx-invalid")).toBe(false);
    });

    it("marca inválido incluso cuando el navegador cancela `submit` antes de dispararlo", () => {
        // Bug real, encontrado con un navegador de verdad (Chromium vía
        // Playwright) contra un `nexa preview` real: al enviar un
        // formulario con un campo `required` vacío, el navegador corre
        // su propia validación de restricciones ANTES de disparar
        // `submit` — si falla, cancela el envío entero y `submit` nunca
        // llega a dispararse (dispara `invalid` en el campo en su
        // lugar). Los demás tests de este archivo usan
        // `form.dispatchEvent(new Event("submit"))` directamente, que se
        // salta esa validación nativa por completo — por eso no lo
        // habían detectado. `requestSubmit()` sí la respeta, igual que
        // un clic real en el botón de envío.
        const { form, email } = buildContactForm();
        form.removeAttribute("novalidate");
        initForm(form);

        form.requestSubmit();

        expect(email.classList.contains("nx-invalid")).toBe(true);
        expect(email.getAttribute("aria-invalid")).toBe("true");
    });

    it("dispose() deja de escuchar", () => {
        const { form, email } = buildContactForm();
        const dispose = initForm(form);
        dispose();

        const event = submit(form);
        expect(event.defaultPrevented).toBe(false); // ya no hay validación activa
        expect(email.classList.contains("nx-invalid")).toBe(false);
    });
});
