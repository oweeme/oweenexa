export const seo = { title: t("home.title"), description: t("home.desc"), canonical: `https://oweeme.com/${params.locale}` };
export default function Home() { return (<main><h1>{t("home.title")}</h1><p>{t("home.desc")}</p><a href={`/${params.locale}/blog`}>{t("nav.blog")}</a></main>); }
