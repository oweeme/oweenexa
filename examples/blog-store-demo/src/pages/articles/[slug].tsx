export const load = { url: "/api/articles/:slug" };
export const paths = { url: "/api/articles" };
export const seo = { title: data.title, description: data.title, canonical: `https://oweeme.com/articles/${params.slug}` };

// `{data.html}` se escaparía (se vería el texto con etiquetas): <SafeHtml> y <Markdown> son la forma
// explícita de decir "esto es contenido enriquecido" — saneado al renderizar.
export default function Article() {
    return (
        <article>
            <h1>{data.title}</h1>
            <section id="html"><SafeHtml html={data.html} /></section>
            <section id="md"><Markdown text={data.markdown} /></section>
            <section id="escaped"><p id="as-text">{data.html}</p></section>
        </article>
    );
}
