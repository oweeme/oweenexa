// Modo App + React: se monta con `ref`, igual que Phaser.
import { createRoot } from "react-dom/client";
import { useState } from "react";
function Counter() { const [n, setN] = useState(0); return <button id="app-react-btn" onClick={() => setN(n + 1)}>React en la SPA: {n}</button>; }
export default function mount(el: Element) { const r = createRoot(el); r.render(<Counter />); return () => r.unmount(); }
