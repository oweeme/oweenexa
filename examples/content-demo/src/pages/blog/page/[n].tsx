// Paginación sin parámetros de consulta: /blog/page/1, /blog/page/2…
// `?pages` enumera las páginas que existen; `?perPage=1&page=:n` trae la que toca.
export const paths = { url: "content:/posts?perPage=1&pages" };
export const load = { url: "content:/posts?perPage=1&page=:n" };
export const seo = { title: `Blog — página ${params.n}`, description: "Artículos" };

export default function BlogPage() {
    return (
        <section>
            <h1>Blog — página {params.n}</h1>
            <For each={data}>{(post) => <p><a href={`/blog/${post.slug}`}>{post.title}</a></p>}</For>
            <p><a href="/blog/page/1">1</a> · <a href="/blog/page/2">2</a></p>
        </section>
    );
}
