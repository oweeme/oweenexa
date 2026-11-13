export const load = { url: "content:/posts/:slug" };
export const paths = { url: "content:/posts" };

export const seo = {
    title: `${data.title} — Content demo`,
    description: data.summary,
};

export default function Post() {
    return (
        <article>
            <p><a href="/blog">← Blog</a></p>
            <h1>{data.title}</h1>
            <p class="meta">{data.date}</p>
            <Markdown text={data.body} />
        </article>
    );
}
