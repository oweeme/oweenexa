/**
 * `platform.theme.get()`/`platform.theme.set()` (Fase 43, issue #11) —
 * persiste la preferencia de tema con `platform.storage()` (ya
 * existente, reutilizado tal cual) y la aplica como
 * `document.documentElement.dataset.theme`, el mismo atributo que leen
 * las variantes oscuras de los tokens de `@nexa/ui` (`tokens.css`).
 *
 * Este módulo reaplica la preferencia guardada apenas se evalúa (efecto
 * de nivel de módulo, más abajo) — así, si el toggle de tema en la
 * página usa `data-nexa-strategy="load"` (Fase 5: ese chunk se importa
 * en cada carga de página, no solo al hacer click), el tema elegido
 * vuelve a verse en cada recarga sin que el desarrollador tenga que
 * llamar nada explícitamente. Sin ese chunk cargado (JS deshabilitado,
 * o el toggle nunca se importó) el sitio sigue siendo correcto: cae al
 * `@media (prefers-color-scheme: dark)` de `tokens.css`, nunca se ve un
 * tema roto.
 */

import { createStorage } from "./storage";

export type Theme = "light" | "dark" | "system";

const STORAGE_KEY = "nexa-theme";

function applyTheme(value: Theme): void {
    if (typeof document === "undefined") return;
    if (value === "system") {
        document.documentElement.removeAttribute("data-theme");
    } else {
        document.documentElement.setAttribute("data-theme", value);
    }
}

function readStoredTheme(): Theme {
    try {
        const stored = createStorage().get(STORAGE_KEY);
        return stored === "dark" || stored === "light" ? stored : "system";
    } catch {
        return "system";
    }
}

export const theme = {
    get(): Theme {
        return readStoredTheme();
    },
    set(value: Theme): void {
        createStorage().set(STORAGE_KEY, value);
        applyTheme(value);
    },
};

applyTheme(readStoredTheme());
