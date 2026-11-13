export default function Card(props: { title: string | (() => string); children?: unknown }) {
    return (
        <section class="card">
            <h2>{props.title}</h2>
            {props.children}
        </section>
    );
}
