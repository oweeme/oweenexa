import { state, navigate } from "@nexa/app";
import { user, api } from "../session";

export default function Login() {
    const name = state("");
    const error = state("");

    async function enter() {
        try {
            const me = await api.post<{ name: string }>("/login", { name: name.value });
            user.value = me.name;
            navigate("/");
        } catch (e) {
            error.value = String(e);
        }
    }

    return (
        <main>
            <h1>Panel</h1>
            {/* `value={name}`: enlace en los dos sentidos con una señal. */}
            <input id="name" placeholder="Tu nombre" value={name} />
            <button id="enter" onClick={enter}>Entrar</button>
            <p id="error">{error}</p>
        </main>
    );
}
