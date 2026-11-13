export default function Layout() {
    return (
        <div class="shell">
            <nav>
                <a href="/">Inicio</a>
                <a href="/blog">Blog</a>
                <a href="/docs">Documentación</a>
            </nav>
            <main data-nexa-slot></main>
        </div>
    );
}
