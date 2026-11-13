import type { Child } from "@nexa/app";

export default function Card(props: { title: string | (() => string); children?: Child | Child[] }) {
    return (
        <section class="card">
            <h2>{props.title}</h2>
            {props.children}
        </section>
    );
}
