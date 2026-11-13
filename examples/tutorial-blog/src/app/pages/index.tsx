import { state, For, Show, navigate } from "@nexa/app";
import { user, api } from "../session";

interface Post { slug: string; title: string; date: string }

export default function Dashboard() {
    const posts = state<Post[]>([]);
    const title = state("");
    const body = state("");
    const saved = state("");

    const load = () => api.get<Post[]>("/posts").then((list) => (posts.value = list)).catch(() => {});
    load();

    async function create() {
        if (!title.value.trim()) return;
        const post = await api.post<Post>("/admin/posts", { title: title.value, body: body.value });
        saved.value = post.slug;
        title.value = "";
        body.value = "";
        await load();
    }

    async function logout() {
        await api.post("/logout");
        user.value = null;
        navigate("/login");
    }

    return (
        <main>
            <h1>Hola, {user}</h1>
            <button id="logout" onClick={logout}>Salir</button>

            <h2>Nuevo artículo</h2>
            <input id="title" placeholder="Título" value={title} />
            <textarea id="body" placeholder="Contenido en Markdown" value={body}></textarea>
            <button id="create" onClick={create}>Publicar</button>
            {/* `Show`: se muestra solo cuando hay algo guardado. */}
            <Show when={saved}>
                <p id="saved">Publicado: <a href={() => `/es/blog/${saved.value}`} data-nexa-reload>{saved}</a></p>
            </Show>

            <h2>Artículos ({() => posts.value.length})</h2>
            <ul id="list">
                <For each={posts}>{(p) => <li>{p.title} <small>({p.date})</small></li>}</For>
            </ul>
        </main>
    );
}
