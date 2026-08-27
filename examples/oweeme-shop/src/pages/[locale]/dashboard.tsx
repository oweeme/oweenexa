export const seo = {
    title: "Dashboard",
    canonical: `/${params.locale}/dashboard`
};

export default function DashboardPage() {
    return (
        <main>
            <h1>Dashboard</h1>
            {/*
              Isla interactiva (Fase 16): a diferencia de las páginas
              públicas de esta tienda, un dashboard no es superficie
              SEO — por eso el fallback es un esqueleto simple, no
              contenido indexable. Lo que monta aquí es un componente
              Vue 3 real (src/islands/Dashboard.ts), vía
              @nexa/vue-island — prueba directa de que Nexa puede
              incrustar el mismo tipo de panel administrativo (tipo
              SDLC/Trello) que ya existe en un proyecto Vue real, sin
              reescribirlo.
            */}
            <div
                class="nx-card"
                data-nexa-island="dashboardIsland"
                data-nexa-strategy="load"
                data-nexa-props={{
                    tasks: [
                        { id: 1, title: "Revisar pedidos pendientes", done: false },
                        { id: 2, title: "Actualizar catálogo", done: true },
                        { id: 3, title: "Responder mensajes de contacto", done: false }
                    ]
                }}
            >
                <p>Cargando panel…</p>
            </div>
        </main>
    );
}
