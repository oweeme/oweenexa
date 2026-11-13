import mount from "reactMount";
import { onCleanup } from "@nexa/app";
export default function Page() {
    let dispose: void | (() => void);
    onCleanup(() => typeof dispose === "function" && dispose());
    return <main><h1>SPA de Nexa</h1><div id="react-in-app" ref={(el: Element) => queueMicrotask(() => (dispose = mount(el)))}></div></main>;
}
