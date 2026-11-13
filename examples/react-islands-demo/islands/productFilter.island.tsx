// Una isla escrita con REACT: el módulo exporta mount(el, props), igual que cualquier isla.
import { useState } from "react";
import { createRoot } from "react-dom/client";

type Group = { name: string; members: { name: string; role: string }[] };

function Filter({ groups }: { groups: Group[] }) {
    const [q, setQ] = useState("");
    const [clicks, setClicks] = useState(0);
    const shown = groups.map((g) => ({ ...g, members: g.members.filter((m) => m.name.toLowerCase().includes(q.toLowerCase())) }));
    return (
        <div id="react-root">
            <input id="react-q" value={q} onChange={(e) => setQ(e.target.value)} placeholder="filtrar (React)" />
            <button id="react-btn" onClick={() => setClicks(clicks + 1)}>clics: {clicks}</button>
            <ul id="react-list">
                {shown.flatMap((g) => g.members.map((m) => <li key={g.name + m.name}>{g.name}: {m.name}</li>))}
            </ul>
        </div>
    );
}

export default function mount(el: Element, props: Record<string, unknown>) {
    const root = createRoot(el);
    root.render(<Filter groups={props.groups as Group[]} />);
    return () => root.unmount();
}
