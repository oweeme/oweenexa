// Un layout ANIDADO: envuelve solo las páginas de src/pages/docs/ (dentro del src/layout.tsx global).
export default function DocsLayout() {
    return (
        <div class="docs">
            <aside class="docs-nav">
                <strong>Documentación</strong>
                <a href="/docs/primeros-pasos">Primeros pasos</a>
                <a href="/docs/guia">La guía</a>
                <a href="/docs/guia/instalar">Instalar</a>
            </aside>
            <div class="docs-body" data-nexa-slot></div>
        </div>
    );
}
