import { createApi, persisted, navigate } from "@nexa/app";

// El estado de la UI (el nombre) sobrevive a recargar la página. La credencial real es la cookie `sid`
// (HttpOnly) que pone tu backend: este código nunca la ve.
export const user = persisted<string | null>("tutorial.user", null);

// `/api` es del mismo origen gracias a `[proxy]` en nexa.toml: sin CORS, y la cookie viaja sola.
export const api = createApi({
    baseURL: "/api",
    // Si la sesión caduca en el backend (401), limpiamos la UI y volvemos al login.
    onUnauthorized: () => {
        user.value = null;
        navigate("/login", { replace: true });
    },
});
