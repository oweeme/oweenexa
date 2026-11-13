import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { initForm } from "../src";

const flush = () => new Promise((r) => setTimeout(r, 0));

function build(html: string): HTMLFormElement {
    document.body.innerHTML = `<form data-nexa-form novalidate>${html}<button type="submit">Ok</button></form>`;
    return document.querySelector("form") as HTMLFormElement;
}
const field = (form: HTMLFormElement, name: string) => form.elements.namedItem(name) as HTMLInputElement;
function submit(form: HTMLFormElement): Event {
    const e = new Event("submit", { bubbles: true, cancelable: true });
    form.dispatchEvent(e);
    return e;
}
function type(el: HTMLInputElement, value: string) {
    el.value = value;
    el.dispatchEvent(new Event("input", { bubbles: true }));
}

describe("data-nexa-match (entre campos)", () => {
    it("bloquea el envío si «confirmar» no coincide y muestra el mensaje propio", () => {
        const form = build(`
            <input name="pass" type="password" /><input name="again" type="password" data-nexa-match="pass" data-nexa-message-match="No coinciden" />
            <span data-nexa-error-for="again"></span>`);
        initForm(form);
        type(field(form, "pass"), "secreto");
        type(field(form, "again"), "otro");
        const e = submit(form);
        expect(e.defaultPrevented).toBe(true);
        expect(form.querySelector("[data-nexa-error-for=again]")!.textContent).toBe("No coinciden");
        type(field(form, "again"), "secreto");
        expect(submit(form).defaultPrevented).toBe(false);
        expect(form.querySelector("[data-nexa-error-for=again]")!.textContent).toBe("");
    });

    it("al cambiar el campo original se revalida el «confirmar» que ya se tocó", () => {
        const form = build(`<input name="pass" /><input name="again" data-nexa-match="pass" /><span data-nexa-error-for="again"></span>`);
        initForm(form);
        type(field(form, "pass"), "a");
        type(field(form, "again"), "a");
        submit(form); // marca touched
        expect(field(form, "again").validity.valid).toBe(true);
        type(field(form, "pass"), "ab");
        expect(field(form, "again").classList.contains("nx-invalid")).toBe(true);
    });
});

describe("data-nexa-remote (contra tu API)", () => {
    beforeEach(() => vi.stubGlobal("fetch", vi.fn()));
    afterEach(() => vi.unstubAllGlobals());
    const reply = (json: unknown, ok = true) => (fetch as unknown as ReturnType<typeof vi.fn>).mockResolvedValue({ ok, json: async () => json });

    it("pregunta al salir del campo y muestra el mensaje de la API", async () => {
        reply({ ok: false, message: "Ya existe" });
        const form = build(`<input name="user" data-nexa-remote="/api/libre?u={value}" /><span data-nexa-error-for="user"></span>`);
        initForm(form);
        field(form, "user").value = "ada lovelace";
        field(form, "user").dispatchEvent(new Event("blur"));
        await flush();
        expect(fetch).toHaveBeenCalledWith("/api/libre?u=ada%20lovelace", expect.anything());
        expect(form.querySelector("[data-nexa-error-for=user]")!.textContent).toBe("Ya existe");
        expect(field(form, "user").classList.contains("nx-invalid")).toBe(true);
    });

    it("al enviar espera a la API: si dice que no, no se envía; si dice que sí, se envía solo", async () => {
        const form = build(`<input name="user" data-nexa-remote="/api/libre?u={value}" /><span data-nexa-error-for="user"></span>`);
        const requestSubmit = vi.fn();
        form.requestSubmit = requestSubmit;
        initForm(form);

        reply({ ok: false, message: "Ocupado" });
        field(form, "user").value = "ada";
        expect(submit(form).defaultPrevented).toBe(true);
        await flush();
        expect(requestSubmit).not.toHaveBeenCalled();
        expect(form.querySelector("[data-nexa-error-for=user]")!.textContent).toBe("Ocupado");

        reply({ ok: true });
        type(field(form, "user"), "grace");
        expect(submit(form).defaultPrevented).toBe(true); // espera a la API
        await flush();
        expect(requestSubmit).toHaveBeenCalledTimes(1); // ya verificado: se reenvía
        // y el reenvío no vuelve a preguntar
        expect(submit(form).defaultPrevented).toBe(false);
        expect(fetch).toHaveBeenCalledTimes(2);
    });

    it("si la API falla o no responde, NO se bloquea al usuario", async () => {
        (fetch as unknown as ReturnType<typeof vi.fn>).mockRejectedValue(new Error("red caída"));
        const form = build(`<input name="user" data-nexa-remote="/api/libre?u={value}" />`);
        const requestSubmit = vi.fn();
        form.requestSubmit = requestSubmit;
        initForm(form);
        field(form, "user").value = "ada";
        submit(form);
        await flush();
        expect(requestSubmit).toHaveBeenCalled();
        expect(field(form, "user").validity.valid).toBe(true);
    });

    it("un campo vacío no consulta, y una respuesta vieja (el usuario siguió escribiendo) se ignora", async () => {
        const form = build(`<input name="user" data-nexa-remote="/api/libre?u={value}" />`);
        initForm(form);
        field(form, "user").dispatchEvent(new Event("blur"));
        await flush();
        expect(fetch).not.toHaveBeenCalled();
        let resolveFirst!: (v: unknown) => void;
        (fetch as unknown as ReturnType<typeof vi.fn>).mockReturnValueOnce(new Promise((r) => (resolveFirst = r)));
        field(form, "user").value = "a";
        field(form, "user").dispatchEvent(new Event("blur"));
        field(form, "user").value = "ab"; // siguió escribiendo
        resolveFirst({ ok: true, json: async () => ({ ok: false, message: "vieja" }) });
        await flush();
        expect(field(form, "user").validity.valid).toBe(true);
    });
});
