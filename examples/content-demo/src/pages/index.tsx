export const seo = { title: "Content demo", description: "Un sitio sin backend: el contenido vive en src/content/." };

export default function Home() {
    return (
        <section>
            <h1>Un sitio sin backend</h1>
            <p>Los artículos y la documentación son archivos Markdown de <code>src/content/</code>.</p>
            <div data-nexa-island="counter" data-nexa-strategy="load" data-nexa-props={{ label: "Clics" }}></div>
        </section>
    );
}
