export default function Card(props) {
    return (
        <section class="card">
            <h2>{props.title}</h2>
            {props.children}
        </section>
    );
}
