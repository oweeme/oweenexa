// Se envuelve TODA página con esto. El único elemento `data-nexa-slot` (vacío) es donde va la página.
export const head = {
    stylesheets: ["/site.css"],
};

export default function Layout() {
    return (
        <div class="shell">
            <header>
                <a class="brand" href={`/${params.locale}`}>{t("site.name")}</a>
                <nav>
                    <a href={`/${params.locale}`}>{t("nav.home")}</a>
                    <a href={`/${params.locale}/blog`}>{t("nav.blog")}</a>
                    <a href="/es" hreflang="es">ES</a>
                    <a href="/en" hreflang="en">EN</a>
                </nav>
            </header>
            <main data-nexa-slot></main>
            <footer>{t("site.tagline")}</footer>
        </div>
    );
}
