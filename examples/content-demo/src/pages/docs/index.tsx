export const load = { url: "content:/docs" };
export const seo = { title: "Documentación", description: "Guías" };

export default function Docs() {
    return (
        <section>
            <h1>Documentación</h1>
            <ul>
                <For each={data}>{(page) => <li><a href={`/docs/${page.path}`}>{page.title}</a></li>}</For>
            </ul>
        </section>
    );
}
