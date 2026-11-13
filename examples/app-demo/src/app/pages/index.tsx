import { state, For, Show, navigate } from "@nexa/app";
import { session, api } from "../session";
import Card from "../components/Card";

interface User { id: number; name: string }

export default function Users() {
    const users = state<User[]>([]);
    const filter = state("");
    const loading = state(true);
    const visible = () => users.value.filter((u) => u.name.toLowerCase().includes(filter.value.toLowerCase()));

    api.get<User[]>("/users").then((list) => {
        users.value = list;
        loading.value = false;
    }).catch(() => (loading.value = false));

    async function logout() {
        await api.post("/logout");
        session.user.value = null;
        navigate("/login");
    }

    return (
        <Card title={() => `Usuarios (${visible().length})`}>
            <p id="who">Sesión: {session.user}</p>
            <button id="logout" onClick={logout}>Salir</button>
            <input id="filter" placeholder="filtrar" value={filter} />
            <Show when={loading} fallback={
                <ul id="list">
                    <For each={visible} fallback={<li>Sin resultados</li>}>
                        {(u) => <li><a href={`/app/users/${u.id}`}>{u.name}</a></li>}
                    </For>
                </ul>
            }>
                <p>Cargando…</p>
            </Show>
        </Card>
    );
}
