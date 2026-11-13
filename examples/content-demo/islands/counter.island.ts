// Una isla que importa un módulo propio y un CSS: `nexa` los empaqueta solo (esbuild).
import { label } from "./format";
import "./counter.css";

export default function mount(el: Element, props: { label?: string }) {
    let n = 0;
    const button = document.createElement("button");
    button.className = "counter";
    const paint = () => (button.textContent = label(props.label ?? "Clics", n));
    paint();
    button.addEventListener("click", () => { n++; paint(); });
    el.replaceChildren(button);
    return () => button.remove();
}
