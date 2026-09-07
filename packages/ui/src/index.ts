import { closeDialog, openDialog } from "./dialog";
import { notify } from "./toast";

export { closeDialog, openDialog } from "./dialog";
export { notify } from "./toast";
export type { ToastHandle, ToastOptions, ToastVariant } from "./toast";

/**
 * Forma agrupada (Fase 16-adjacent fix), igual que `platform`/`stripe`:
 * `nexa-activation` detecta el uso de un identificador vía `<nombre>.`
 * en el código fuente de un handler (`ui.openDialog(...)`), no llamadas
 * bare (`openDialog(...)`) — así que la única forma realmente utilizable
 * desde una página es a través de este objeto.
 */
export const ui = { openDialog, closeDialog, notify };
