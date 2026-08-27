import { effect, state } from "@nexa/reactivity";

/**
 * Isla (Fase 16) escrita a mano con `@nexa/reactivity` — sin ningún
 * framework externo. El fallback SSR (el `<ul>` estático que ya renderiza
 * `index.tsx`) es el contenido real que ve un buscador; esta isla solo
 * reemplaza esa lista por una versión filtrable en el navegador, usando
 * las mismas props (`data.products`) que ya resolvió el servidor — sin
 * volver a pedirlas.
 */
interface Product {
    slug: string;
    name: string;
    price: number;
}

function isProduct(value: unknown): value is Product {
    return (
        !!value &&
        typeof value === "object" &&
        typeof (value as Product).slug === "string" &&
        typeof (value as Product).name === "string" &&
        typeof (value as Product).price === "number"
    );
}

export default function mount(el: Element, props: Record<string, unknown>): () => void {
    const products = Array.isArray(props.products) ? props.products.filter(isProduct) : [];

    const input = document.createElement("input");
    input.type = "search";
    input.className = "nx-input";
    input.placeholder = "Buscar productos…";

    const list = document.createElement("ul");

    el.innerHTML = "";
    el.append(input, list);

    const query = state("");
    const onInput = () => {
        query.value = input.value;
    };
    input.addEventListener("input", onInput);

    const disposeEffect = effect(() => {
        const needle = query.value.trim().toLowerCase();
        const visible = needle ? products.filter((p) => p.name.toLowerCase().includes(needle)) : products;

        list.innerHTML = "";
        for (const product of visible) {
            const item = document.createElement("li");
            item.textContent = `${product.name} — $${product.price}`;
            list.appendChild(item);
        }
        if (visible.length === 0) {
            const empty = document.createElement("li");
            empty.textContent = "Sin resultados.";
            list.appendChild(empty);
        }
    });

    return () => {
        input.removeEventListener("input", onInput);
        disposeEffect();
    };
}
