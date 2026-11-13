// Una isla en TYPESCRIPT PURO (sin ningún framework).
export default function mount(el: Element, props: Record<string, unknown>) {
    const groups = props.groups as { name: string }[];
    const btn = document.createElement("button");
    btn.id = "ts-btn";
    btn.textContent = `TS puro: ${groups.length} grupos`;
    let n = 0;
    btn.addEventListener("click", () => (btn.textContent = `TS puro: ${groups.length} grupos, clic ${++n}`));
    el.replaceChildren(btn);
    return () => btn.remove();
}
