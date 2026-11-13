export const seo = { title: "Phaser + Nexa", description: "Un juego 2D dentro del Modo App de Nexa", canonical: "https://example.com/" };

export default function Home() {
    return (
        <main>
            <h1>Phaser + Nexa</h1>
            <p>Esta portada es HTML real (SEO). El juego vive en la SPA:</p>
            <a href="/app" data-nexa-reload>Jugar</a>
        </main>
    );
}
