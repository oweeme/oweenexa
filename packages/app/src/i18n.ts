import { state } from "@nexa/reactivity";
import { persisted } from "./persisted";
import type { Signal } from "@nexa/reactivity";

/** Un diccionario anidado: `{ home: { title: "Hola" }, cart: { items: { one: "{count} artículo", other: "{count} artículos" } } }`. */
export type Dictionary = { [key: string]: string | Dictionary };

export interface I18nOptions {
    /** Idioma de respaldo (y el inicial si no hay otra señal). Por defecto, el primero de `dictionaries`. */
    fallback?: string;
    /** Clave de `localStorage` donde recordar la elección (por defecto `nexa-locale`; `null` = no recordar). */
    storageKey?: string | null;
}

export interface I18n {
    /** El idioma activo, como señal: lo que lea `t()` dentro de un efecto o de `{() => …}` se actualiza solo. */
    readonly locale: Signal<string>;
    /** Los idiomas disponibles. */
    readonly available: string[];
    /** Traduce `key` (`"home.title"`). Con `{ name }` interpola `{name}`; con `{ count }` elige singular/plural. */
    t(key: string, args?: Record<string, string | number>): string;
    /** Cambia el idioma (ignora uno que no exista). */
    setLocale(locale: string): void;
    /** Formatos con las reglas del idioma activo (`Intl`). */
    number(value: number, options?: Intl.NumberFormatOptions): string;
    currency(value: number, currency: string, options?: Intl.NumberFormatOptions): string;
    date(value: Date | number | string, options?: Intl.DateTimeFormatOptions): string;
}

const PLURAL = ["zero", "one", "two", "few", "many", "other"];

/**
 * Traducciones para el Modo App — lo mismo que `t()` en Nexa Core, pero en el navegador y con cambio de
 * idioma en caliente.
 *
 * ```ts
 * export const i18n = createI18n({ es: { hi: "Hola, {name}" }, en: { hi: "Hello, {name}" } });
 * // en una página:  <h1>{() => i18n.t("hi", { name: user.value })}</h1>
 * //                 <button onClick={() => i18n.setLocale("en")}>EN</button>
 * ```
 *
 * Una clave sin traducir cae al idioma de respaldo, y si tampoco está, devuelve la propia clave (se nota en pantalla).
 */
export function createI18n(dictionaries: Record<string, Dictionary>, options: I18nOptions = {}): I18n {
    const available = Object.keys(dictionaries);
    if (available.length === 0) throw new Error("createI18n: hace falta al menos un idioma");
    const fallback = options.fallback && dictionaries[options.fallback] ? options.fallback : available[0]!;
    const key = options.storageKey === undefined ? "nexa-locale" : options.storageKey;

    const initial = (() => {
        const browser = (globalThis.navigator?.languages ?? [globalThis.navigator?.language]).filter(Boolean) as string[];
        for (const l of browser) {
            const lang = l.toLowerCase();
            const hit = available.find((a) => a.toLowerCase() === lang) ?? available.find((a) => a.toLowerCase() === lang.split("-")[0]);
            if (hit) return hit;
        }
        return fallback;
    })();
    const locale: Signal<string> = key === null ? (state<string>(initial) as Signal<string>) : persisted<string>(key, initial);
    // una elección guardada de un idioma que ya no existe
    if (!dictionaries[locale.value]) locale.value = fallback;

    const lookup = (dict: Dictionary | undefined, path: string): string | Dictionary | undefined => {
        let v: string | Dictionary | undefined = dict;
        for (const seg of path.split(".")) {
            if (typeof v !== "object" || v === null) return undefined;
            v = v[seg];
        }
        return v;
    };

    function t(path: string, args: Record<string, string | number> = {}): string {
        const current = locale.value;
        let found = lookup(dictionaries[current], path);
        let lang = current;
        if (found === undefined && current !== fallback) {
            found = lookup(dictionaries[fallback], path);
            lang = fallback;
        }
        if (found === undefined) return path;
        let text: string | undefined;
        if (typeof found === "string") text = found;
        else {
            // una forma plural: `{ one, other }`
            const n = Number(args.count ?? args.n);
            if (!Number.isFinite(n)) return path;
            const category = new Intl.PluralRules(lang).select(n);
            const pick = (found as Dictionary)[category] ?? (found as Dictionary).other;
            if (typeof pick !== "string") return path;
            if (!Object.keys(found).every((k) => PLURAL.includes(k))) return path;
            text = pick;
        }
        return text.replace(/\{(\w+)\}/g, (whole, name: string) => (name in args ? String(args[name]) : whole));
    }

    return {
        locale,
        available,
        t,
        setLocale(next) {
            if (dictionaries[next]) locale.value = next;
        },
        number: (value, o) => new Intl.NumberFormat(locale.value, o).format(value),
        currency: (value, currency, o) => new Intl.NumberFormat(locale.value, { style: "currency", currency, ...o }).format(value),
        date: (value, o) => new Intl.DateTimeFormat(locale.value, o).format(new Date(value)),
    };
}

