import type { PageProps } from "@nexa/app";
import { session } from "./session";

// Ruta protegida: sin sesión, todo menos /login va a /login.
export function guard(to: { path: string }) {
    if (!session.user.value && to.path !== "/login") return "/login";
    if (session.user.value && to.path === "/login") return "/";
}
