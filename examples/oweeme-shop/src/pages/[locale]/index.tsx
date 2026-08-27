export const load = { url: "/products" };

export const seo = {
    title: t("home.title"),
    description: t("home.subtitle"),
    canonical: `/${params.locale}`
};

export const schema = {
    type: "Organization",
    name: "Oweeme",
    url: `https://oweeme.example/${params.locale}`
};

function shareStore() {
    platform.share({ title: "Oweeme", url: location.href }).catch(() => {
        // No todos los navegadores/entornos soportan la Web Share API —
        // ver la limitación documentada de @nexa/platform.
    });
}

export default function Home() {
    return (
        <main>
            <h1>{t("home.title")}</h1>
            <p>{t("home.subtitle")}</p>
            <nav>
                <a href="/es">Español</a>
                <a href="/en">English</a>
            </nav>
            <div class="nx-card">
                <h2 class="nx-card-title">{t("home.featured")}</h2>
                <a class="nx-btn nx-btn-primary" href={`/${params.locale}/products/iphone-17`}>
                    {t("home.cta")}
                </a>
            </div>
            <button class="nx-btn" onClick={shareStore}>{t("home.share")}</button>
            <a class="nx-btn" href={`/${params.locale}/contact`}>{t("home.contact")}</a>

            {/*
              Isla interactiva (Fase 16): Nexa no tiene composición ni
              bucles todavía, así que el fallback SSR de abajo no se
              genera desde `data.products` — se escribe a mano, igual
              que el resto de esta página (contenido real, indexable,
              con enlaces reales a cada página de producto). Las props
              sí llevan el array completo que devolvió `load()`: la isla
              usa esos mismos datos para el filtro en el cliente, sin
              volver a pedirlos.
            */}
            <section class="nx-card" data-nexa-island="productFilter" data-nexa-props={{ products: data.products }}>
                <h2 class="nx-card-title">{t("home.catalog")}</h2>
                <ul>
                    <li><a href={`/${params.locale}/products/iphone-17`}>iPhone 17 — $999</a></li>
                    <li><a href={`/${params.locale}/products/pixel-10`}>Pixel 10 — $799</a></li>
                </ul>
            </section>
        </main>
    );
}
