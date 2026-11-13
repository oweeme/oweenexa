import { state } from "@nexa/app";
import { api } from "../../session";
import type { PageProps } from "@nexa/app";

export default function UserDetail(props: PageProps) {
    const user = state<{ name: string; email: string } | null>(null);
    api.get<{ name: string; email: string }>(`/users/${props.params.id}`)
        .then((u) => (user.value = u))
        .catch(() => {}); // un 401 ya lo atiende onUnauthorized (session.ts)
    return (
        <article>
            <a href="/app">← volver</a>
            <h1 id="title">{() => user.value?.name ?? "…"}</h1>
            <p id="email">{() => user.value?.email ?? ""}</p>
        </article>
    );
}
