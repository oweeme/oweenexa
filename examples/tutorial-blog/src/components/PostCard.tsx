// Un componente de Nexa Core: NO existe en ejecución. Al compilar, `<PostCard post={p} />` se sustituye
// por este HTML con los props ya puestos (el resultado es el mismo que si lo escribieras a mano).
export default function PostCard(props) {
    return (
        <article class="card">
            <h3><a href={`/${props.locale}/blog/${props.post.slug}`}>{props.post.title}</a></h3>
            <p class="meta">{props.post.date} · {t("blog.by")} {props.post.author}</p>
            <p>{props.post.excerpt}…</p>
        </article>
    );
}
