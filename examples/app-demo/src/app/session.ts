import { createApi, persisted, navigate } from "@nexa/app";

/**
 * Estado de la UI de la sesión (el nombre), persistido entre recargas.
 * La credencial real es la cookie `sid` HttpOnly que pone el backend:
 * este código nunca la ve.
 */
export const session = {
    user: persisted<string | null>("demo.user", null),
};

/** Mismo origen que la app (gracias a `[proxy]`): sin CORS, las cookies viajan solas. */
export const api = createApi({
    baseURL: "/api",
    // Sesión vencida en el backend -> limpiar la UI y volver al login.
    onUnauthorized: () => {
        session.user.value = null;
        navigate("/login", { replace: true });
    },
});
