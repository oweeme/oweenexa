export const seo = { title: "No encontrada", description: "Esta página no existe" };

export default function NotFound() {
    return (
        <section>
            <h1>404 — No encontrada</h1>
            <p>Esta página no existe. <a href="/">Volver al inicio</a>.</p>
        </section>
    );
}
