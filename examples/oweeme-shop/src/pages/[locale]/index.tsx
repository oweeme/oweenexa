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
              Isla interactiva (Fase 16) + iteración real (Fase 30): el
              fallback SSR de abajo ahora sí sale de `data.products` con
              `<For>` — HTML real, indexable, con un <li> por producto de
              verdad (antes de la Fase 30 esto se escribía a mano, con
              solo dos productos hardcodeados). Las props de la isla
              llevan el mismo array completo: el filtro del cliente usa
              esos mismos datos, sin volver a pedirlos.
            */}
            <section class="nx-card" data-nexa-island="productFilter" data-nexa-props={{ products: data.products }}>
                <h2 class="nx-card-title">{t("home.catalog")}</h2>
                <ul>
                    <For each={data.products}>
                        {(product) => (
                            <li>
                                <a href={`/${params.locale}/products/${product.slug}`}>
                                    {product.name}{" "}— ${product.price}
                                </a>
                            </li>
                        )}
                    </For>
                </ul>
            </section>
        </main>
    );
}
