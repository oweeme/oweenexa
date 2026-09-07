/**
 * Toast / snackbar de `@nexa/ui` (Fase 28): notificación efímera *dentro*
 * de la página — no confundir con `platform.notify()`, que es una
 * notificación real del sistema operativo (Capacitor/Notification API).
 * Pensado para dashboards y SPAs: "guardado con éxito", "no se pudo
 * conectar", etc.
 *
 * A diferencia de Button/Input/Card/Dialog, el HTML de un toast no lo
 * escribe el desarrollador — lo crea `notify()` en el momento. Por eso
 * su CSS no pasa por el tree-shaking a nivel de sitio (que solo mira
 * `class="..."` estático en el JSX de cada página): se inyecta una sola
 * vez, en runtime, la primera vez que se llama `notify()` — cero costo
 * si nunca se usa, igual que el resto de Nexa.
 */

export type ToastVariant = "info" | "success" | "warning" | "danger";

export interface ToastOptions {
    message: string;
    variant?: ToastVariant;
    /** ms antes de auto-cerrarse. 0 desactiva el auto-cierre. Default: 4000. */
    duration?: number;
}

export interface ToastHandle {
    /** Cierra el toast ya mismo, aunque su duración todavía no haya pasado. */
    close(): void;
}

const DEFAULT_DURATION = 4000;
const LEAVE_MS = 200;

let container: HTMLElement | null = null;

export function notify(options: ToastOptions): ToastHandle {
    injectStylesOnce();

    const el = buildToastElement(options);
    getContainer().appendChild(el);

    let closed = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    const close = () => {
        if (closed) return;
        closed = true;
        if (timer) clearTimeout(timer);
        el.classList.add("nx-toast-leaving");
        setTimeout(() => el.remove(), LEAVE_MS);
    };

    const duration = options.duration ?? DEFAULT_DURATION;
    if (duration > 0) {
        timer = setTimeout(close, duration);
    }

    el.querySelector(".nx-toast-close")?.addEventListener("click", close);

    return { close };
}

function buildToastElement(options: ToastOptions): HTMLElement {
    const el = document.createElement("div");
    el.className = `nx-toast nx-toast-${options.variant ?? "info"}`;
    el.setAttribute("role", "status");
    el.setAttribute("aria-live", "polite");

    const message = document.createElement("span");
    message.className = "nx-toast-message";
    message.textContent = options.message;
    el.appendChild(message);

    const closeBtn = document.createElement("button");
    closeBtn.type = "button";
    closeBtn.className = "nx-toast-close";
    closeBtn.setAttribute("aria-label", "Cerrar");
    closeBtn.textContent = "×";
    el.appendChild(closeBtn);

    return el;
}

function getContainer(): HTMLElement {
    if (!container || !container.isConnected) {
        container = document.createElement("div");
        container.className = "nx-toast-container";
        document.body.appendChild(container);
    }
    return container;
}

/**
 * Se chequea la presencia real en el DOM (no un booleano en memoria):
 * así se auto-repara si algo más externo llegó a quitar el `<style>`,
 * en vez de quedar convencido para siempre de que ya está.
 */
function injectStylesOnce(): void {
    if (document.head.querySelector("style[data-nexa-ui-toast]")) return;
    const style = document.createElement("style");
    style.setAttribute("data-nexa-ui-toast", "");
    style.textContent = TOAST_CSS;
    document.head.appendChild(style);
}

const TOAST_CSS = `
.nx-toast-container {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 2147483647;
    max-width: 320px;
}
.nx-toast {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: var(--nx-radius-md, 0.5rem);
    background: var(--nx-color-surface, #ffffff);
    color: var(--nx-color-text, #111827);
    box-shadow: var(--nx-shadow-md, 0 4px 12px rgba(0, 0, 0, 0.15));
    font-family: var(--nx-font-body, system-ui, sans-serif);
    font-size: 0.9rem;
    border-left: 4px solid #6b7280;
    opacity: 1;
    transform: translateY(0);
    transition: opacity ${LEAVE_MS}ms ease, transform ${LEAVE_MS}ms ease;
}
.nx-toast-leaving { opacity: 0; transform: translateY(4px); }
.nx-toast-info { border-left-color: var(--nx-color-primary, #2563eb); }
.nx-toast-success { border-left-color: #16a34a; }
.nx-toast-warning { border-left-color: #d97706; }
.nx-toast-danger { border-left-color: var(--nx-color-danger, #dc2626); }
.nx-toast-message { flex: 1; }
.nx-toast-close {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    color: inherit;
    opacity: 0.6;
    padding: 0;
}
.nx-toast-close:hover { opacity: 1; }
`;
