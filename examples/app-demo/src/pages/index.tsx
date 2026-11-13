export const seo = { title: "Demo Nexa", description: "Sitio con SEO y una SPA en /app" };
export default function Home() {
    return (
        <main>
            <h1>Landing con SEO</h1>
            <p>HTML real, indexable.</p>
            <a href="/app" data-nexa-reload>Entrar a la app</a>
        </main>
    );
}
