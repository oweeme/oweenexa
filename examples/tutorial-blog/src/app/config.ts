import { user } from "./session";

// Rutas protegidas: sin sesión, todo menos /login lleva a /login (y al revés).
// OJO: esto protege la INTERFAZ. Lo que protege los datos es que tu API responda 401.
export function guard(to: { path: string }) {
    if (!user.value && to.path !== "/login") return "/login";
    if (user.value && to.path === "/login") return "/";
}
