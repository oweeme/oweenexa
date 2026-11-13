import Card from "../components/Card";
import { Tag, Button } from "../components/ui";

export const load = { url: "/api/catalog" };
export const seo = { title: "Catálogo", description: "Página Core con componentes", canonical: "https://example.com/catalog" };

function saludar() {
    console.log("saludo desde un componente");
}

export default function Catalog() {
    return (
        <main>
            <h1>{data.title}</h1>
            <For each={data.groups}>
                {(group) => (
                    <Card title={group.name}>
                        <ul>
                            <For each={group.members}>
                                {(m) => (
                                    <li>
                                        <Tag text={m.role} /> {m.name}
                                    </li>
                                )}
                            </For>
                        </ul>
                        <Button onPress={saludar}>Saludar</Button>
                    </Card>
                )}
            </For>
        </main>
    );
}
