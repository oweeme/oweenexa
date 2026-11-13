// Una ruta DINÁMICA: /es/blog/hola-nexa, /en/blog/hola-nexa, ...
export const load = { url: "/api/posts/:slug?lang=:locale" }; // los datos de ESTA página
export const paths = { url: "/api/posts" };                    // qué páginas pre-renderizar (los idiomas se añaden solos)

export const seo = {
    title: `${data.title} — ${t("site.name")}`,
    description: data.excerpt,
    canonical: `https://example.com/${params.locale}/blog/${params.slug}`,
};

// Datos estructurados para buscadores (JSON-LD).
export const schema = {
    type: "Article",
    headline: data.title,
    datePublished: data.date,
    author: { type: "Person", name: data.author },
};

export default function Post() {
    return (
        <article>
            <p><a href={`/${params.locale}/blog`}>{t("blog.back")}</a></p>
            <h1>{data.title}</h1>
            <p class="meta">{data.date} · {t("blog.by")} {data.author}</p>
            {/* El cuerpo es Markdown de la API: <Markdown> lo convierte a HTML y lo SANEA (sin <script>, sin javascript:). */}
            <Markdown text={data.body} />
        </article>
    );
}
