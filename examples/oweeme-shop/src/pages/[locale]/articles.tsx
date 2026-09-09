export const seo = {
    title: t("articles.title"),
    description: t("articles.subtitle"),
    canonical: `/${params.locale}/articles`
};

// Vista previa en vivo de Markdown mientras se escribe un artículo — el
// caso real que menciona el issue #21 ("marked, ya lo usás para
// artículos"). `marked` corre 100% en el cliente (Nexa nunca ejecuta
// JS en build time): esto NO reemplaza el HTML servidor-renderizado de
// un artículo ya publicado (eso seguiría siendo `data.body` resuelto
// por `load()`, como cualquier otro texto), es la herramienta de
// autoría. `innerHTML` acá es seguro: es la vista previa de quien
// escribe su propio texto, no contenido de un tercero.
function updatePreview() {
    const text = document.querySelector("[data-md-input]").value;
    document.querySelector("[data-md-preview]").innerHTML = marked.parse(text);
}

export default function Articles() {
    return (
        <main class="nx-stack">
            <h1>{t("articles.title")}</h1>
            <p>{t("articles.subtitle")}</p>
            <div class="nx-grid nx-grid-cols-2">
                <div class="nx-card">
                    <h2 class="nx-card-title">{t("articles.writeLabel")}</h2>
                    <textarea class="nx-input" data-md-input onInput={updatePreview} rows="10">
                        {t("articles.placeholder")}
                    </textarea>
                </div>
                <div class="nx-card">
                    <h2 class="nx-card-title">{t("articles.previewLabel")}</h2>
                    <div data-md-preview></div>
                </div>
            </div>
        </main>
    );
}
