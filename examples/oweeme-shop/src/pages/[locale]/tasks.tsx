export const seo = {
    title: "Tasks (lightweight components)",
    canonical: `/${params.locale}/tasks`
};

export default function TasksPage() {
    return (
        <main>
            <h1>Tasks</h1>
            {/*
              Mismo widget que /dashboard (panel de tareas con checkbox
              y contador de pendientes), pero con el modelo de
              componentes liviano de @nexa/reactivity (Fase 50, issue
              #19) en vez de un componente Vue 3 real — comparación
              directa de peso de bundle entre las dos formas de
              construir una isla chica con estado real.
            */}
            <div
                class="nx-card"
                data-nexa-island="taskListIsland"
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
