// `[...path]` recoge uno o más segmentos: /docs/guia/instalar → path = "guia/instalar".
export const load = { url: "content:/docs/:path" };
export const paths = { url: "content:/docs" };

export const seo = { title: `${data.title} — Documentación`, description: data.title };

export default function DocPage() {
    return (
        <article>
            <p><a href="/docs">← Documentación</a></p>
            <h1>{data.title}</h1>
            <Markdown text={data.body} />
        </article>
    );
}
