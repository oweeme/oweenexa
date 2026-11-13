// Una ISLA escrita en TypeScript puro. Nexa la llama con `mount(elemento, props)`.
// Mejora una lista que YA existe en el HTML: filtra los artículos mientras escribes.
export default function mount(el: Element, props: Record<string, unknown>) {
    const input = document.createElement("input");
    input.type = "search";
    input.placeholder = String(props.placeholder ?? "Buscar…");
    input.className = "search";
    el.replaceChildren(input);

    const onInput = () => {
        const q = input.value.trim().toLowerCase();
        document.querySelectorAll<HTMLElement>("#post-list .card").forEach((card) => {
            card.hidden = q !== "" && !card.textContent!.toLowerCase().includes(q);
        });
    };
    input.addEventListener("input", onInput);

    // Lo que devuelves se llama al desmontar la isla.
    return () => input.removeEventListener("input", onInput);
}
