import { state, navigate } from "@nexa/app";
import { platform } from "platform";
import { session, api } from "../session";
import Card from "../components/Card";

export default function Login() {
    const name = state("");
    const error = state("");
    // Solo dentro de la app de escritorio (Tauri): el puente a Rust.
    const desktop = state("");
    if (platform.isTauri()) {
        platform.native
            .appInfo()
            .then((i) => (desktop.value = `Escritorio: ${i.name} v${i.version} — ${i.os}/${i.arch}`))
            .catch((e) => (desktop.value = `Error del puente: ${e}`));
    }

    async function enter() {
        try {
            const me = await api.post<{ name: string }>("/login", { name: name.value });
            session.user.value = me.name;
            navigate("/");
        } catch (e) {
            error.value = String(e);
        }
    }

    return (
        <Card title="Iniciar sesión">
            <input id="name" placeholder="tu nombre" value={name} />
            <button id="enter" onClick={enter}>Entrar</button>
            <p id="preview">Hola, {() => name.value || "…"}</p>
            <p id="error">{error}</p>
            <p id="desktop">{desktop}</p>
        </Card>
    );
}
