export const load = { url: "/api/blog/:author/:slug" };
export const paths = { url: "/api/blog" };
export const seo = { title: `${data.title} — ${params.author}`, description: data.body, canonical: `https://oweeme.com/blog/${params.author}/${params.slug}` };
export default function Post() {
    return (<article><h1>{data.title}</h1><p>por {params.author}</p><p id="body">{data.body}</p></article>);
}
