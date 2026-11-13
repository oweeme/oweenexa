export const load = { url: "content:/posts" };
export const seo = { title: "Blog", description: "Artículos" };

export default function Blog() {
    return (
        <section>
            <h1>Blog</h1>
            <For each={data}>
                {(post) => (
                    <article class="card">
                        <h2><a href={`/blog/${post.slug}`}>{post.title}</a></h2>
                        <p class="meta">{post.date}</p>
                    </article>
                )}
            </For>
        </section>
    );
}
