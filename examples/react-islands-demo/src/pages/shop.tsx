export const load = { url: "/api/catalog" };
export const seo = { title: "Tienda", description: "Catálogo desde una API", canonical: "https://example.com/shop" };

export default function Shop() {
    return (
        <main>
            <h1>{data.title}</h1>
            <section id="static-list">
                <For each={data.groups}>
                    {(g) => (
                        <article>
                            <h2>{g.name}</h2>
                            <For each={g.members}>{(m) => <p>{m.name} — {m.role}</p>}</For>
                        </article>
                    )}
                </For>
            </section>
            <div data-nexa-island="productFilter" data-nexa-strategy="load" data-nexa-props={{ groups: data.groups }}>
                <p>(React se está cargando…)</p>
            </div>
            <div data-nexa-island="tsIsland" data-nexa-strategy="load" data-nexa-props={{ groups: data.groups }}></div>
        </main>
    );
}
