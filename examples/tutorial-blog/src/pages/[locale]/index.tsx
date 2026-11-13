import PostCard from "../../components/PostCard";

// Los datos vienen de TU API, al compilar. `:locale` se sustituye por el idioma de la página.
export const load = { url: "/api/posts?lang=:locale" };

export const seo = {
    title: `${t("site.name")} — ${t("home.title")}`,
    description: t("site.tagline"),
    canonical: `https://example.com/${params.locale}`,
};

export default function Home() {
    return (
        <section>
            <h1>{t("home.title")}</h1>
            <h2>{t("home.latest")}</h2>
            <For each={data}>{(post) => <PostCard post={post} locale={params.locale} />}</For>
        </section>
    );
}
