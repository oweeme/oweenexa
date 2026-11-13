import PostCard from "../../../components/PostCard";

export const load = { url: "/api/posts?lang=:locale" };

export const seo = {
    title: `${t("blog.title")} — ${t("site.name")}`,
    description: t("blog.title"),
    canonical: `https://example.com/${params.locale}/blog`,
};

export default function Blog() {
    return (
        <section>
            <h1>{t("blog.title")}</h1>

            {/* La ISLA: mejora progresiva. Sin JavaScript la lista de abajo sigue siendo HTML completo. */}
            <div data-nexa-island="search" data-nexa-strategy="load" data-nexa-props={{ placeholder: t("blog.search") }}></div>

            <div id="post-list">
                <For each={data}>{(post) => <PostCard post={post} locale={params.locale} />}</For>
            </div>
        </section>
    );
}
