export const load = { url: "/products/:slug" };

export const seo = {
    title: `${data.name} | Oweeme`,
    description: data.description,
    canonical: `/${params.locale}/products/${params.slug}`,
    openGraph: { title: data.name, image: data.image }
};

export const schema = {
    type: "Product",
    name: data.name,
    description: data.description,
    image: data.image,
    offers: {
        type: "Offer",
        price: data.price,
        priceCurrency: "USD",
        availability: "https://schema.org/InStock",
        url: `https://oweeme.example/${params.locale}/products/${params.slug}`
    }
};

async function checkout() {
    const client = await stripe.load("pk_test_51ExampleReferenceOnlyDoNotUseInProduction");
    if (!client) {
        console.warn("[oweeme-shop] Stripe no cargó — revisa la clave pública.");
        return;
    }
    // En una tienda real esto llamaría a un endpoint del backend que
    // crea una Checkout Session real y devuelve su id.
    console.log("[oweeme-shop] cliente de Stripe listo:", client);
}

function shareProduct() {
    platform.share({ title: data.name, url: location.href }).catch(() => {});
}

export default function ProductPage() {
    return (
        <article>
            <h1>{data.name}</h1>
            <img src={data.image} alt={data.name} />
            <p>{data.description}</p>
            <p>${data.price}</p>
            <button class="nx-btn nx-btn-primary" onClick={checkout}>{t("product.buy")}</button>
            <button class="nx-btn" onClick={shareProduct}>{t("product.share")}</button>
        </article>
    );
}
