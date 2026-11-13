import Phaser from "phaser";
import { state, onCleanup } from "@nexa/app";

export default function Game() {
    const frames = state(0);
    const hits = state(0);
    let game: Phaser.Game | undefined;

    class Main extends Phaser.Scene {
        box!: Phaser.GameObjects.Rectangle;
        create() {
            this.box = this.add.rectangle(60, 200, 60, 60, 0xe94560).setInteractive();
            // click sobre el cuadrado: suma un punto (entrada del juego -> señal de Nexa)
            this.box.on("pointerdown", () => hits.value++);
            this.add.text(10, 10, "Phaser dentro de Nexa", { color: "#ffffff" });
        }
        update() {
            this.box.x = (this.box.x + 3) % 780;
            frames.value++; // estado del juego -> HTML reactivo de Nexa
        }
    }

    // Se monta cuando el contenedor ya está en el DOM.
    const mount = (el: HTMLElement) =>
        queueMicrotask(() => {
            game = new Phaser.Game({ type: Phaser.CANVAS, width: 800, height: 400, parent: el, backgroundColor: "#1a1a2e", scene: Main });
            (window as any).__game = game;
        });
    onCleanup(() => game?.destroy(true));

    return (
        <main>
            <h1>Juego con Phaser</h1>
            <p id="stats">Frames: {frames} · Clics en el cuadrado: {hits}</p>
            <div id="game" ref={mount}></div>
        </main>
    );
}
