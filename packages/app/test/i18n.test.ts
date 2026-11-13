import { beforeEach, describe, expect, it } from "vitest";
import { createI18n } from "../src/i18n";

const dictionaries = {
    es: { hi: "Hola, {name}", only: { es: "solo en español" }, cart: { items: { one: "{count} artículo", other: "{count} artículos" } } },
    en: { hi: "Hello, {name}", cart: { items: { one: "{count} item", other: "{count} items" } } },
    ru: { cart: { items: { one: "{count} товар", few: "{count} товара", many: "{count} товаров", other: "{count} товара" } } },
};

beforeEach(() => localStorage.clear());

describe("createI18n", () => {
    it("traduce, interpola y cambia de idioma en caliente", () => {
        const i18n = createI18n(dictionaries, { fallback: "es" });
        i18n.setLocale("es");
        expect(i18n.t("hi", { name: "Ada" })).toBe("Hola, Ada");
        i18n.setLocale("en");
        expect(i18n.t("hi", { name: "Ada" })).toBe("Hello, Ada");
        expect(i18n.locale.value).toBe("en");
    });

    it("una clave sin traducir cae al idioma de respaldo, y si no existe devuelve la clave", () => {
        const i18n = createI18n(dictionaries, { fallback: "es" });
        i18n.setLocale("en");
        expect(i18n.t("only.es")).toBe("solo en español");
        expect(i18n.t("no.existe")).toBe("no.existe");
    });

    it("elige la forma plural con las reglas del idioma", () => {
        const i18n = createI18n(dictionaries, { fallback: "es" });
        i18n.setLocale("es");
        expect(i18n.t("cart.items", { count: 1 })).toBe("1 artículo");
        expect(i18n.t("cart.items", { count: 0 })).toBe("0 artículos");
        i18n.setLocale("ru");
        expect(i18n.t("cart.items", { count: 1 })).toBe("1 товар");
        expect(i18n.t("cart.items", { count: 3 })).toBe("3 товара");
        expect(i18n.t("cart.items", { count: 5 })).toBe("5 товаров");
        // sin `count` no se puede elegir una forma
        expect(i18n.t("cart.items")).toBe("cart.items");
    });

    it("ignora un idioma que no existe y recuerda el elegido", () => {
        const i18n = createI18n(dictionaries, { fallback: "es" });
        i18n.setLocale("en");
        i18n.setLocale("klingon");
        expect(i18n.locale.value).toBe("en");
        // otra instancia (una recarga) recupera la elección
        expect(createI18n(dictionaries, { fallback: "es" }).locale.value).toBe("en");
        // y un idioma guardado que ya no existe vuelve al de respaldo
        localStorage.setItem("nexa-locale", JSON.stringify("xx"));
        expect(createI18n(dictionaries, { fallback: "es" }).locale.value).toBe("es");
    });

    it("formatea números, monedas y fechas con el idioma activo", () => {
        const i18n = createI18n(dictionaries, { fallback: "es", storageKey: null });
        i18n.setLocale("en");
        expect(i18n.number(1234.5)).toBe("1,234.5");
        expect(i18n.currency(9.5, "USD")).toBe("$9.50");
        expect(i18n.date(Date.UTC(2025, 2, 1), { timeZone: "UTC", dateStyle: "medium" })).toBe("Mar 1, 2025");
        i18n.setLocale("es");
        expect(i18n.number(1234.5)).toBe("1234,5");
    });

    it("el texto interpolado no se interpreta como HTML: lo escapa quien lo pinta (h() usa nodos de texto)", () => {
        const i18n = createI18n({ es: { hi: "Hola, {name}" } });
        expect(i18n.t("hi", { name: "<b>x</b>" })).toBe("Hola, <b>x</b>");
    });

    it("sin idiomas es un error claro", () => {
        expect(() => createI18n({})).toThrow();
    });
});
