import { closeDialog, openDialog } from "./dialog";
import { alert, confirm } from "./confirm-dialog";
import { closeDrawer, openDrawer } from "./drawer";
import { notify } from "./toast";
import { selectTab } from "./tabs";
import { toggleAccordionItem } from "./accordion";
import { selectDropdownOption, toggleDropdown } from "./dropdown";
import { filterTable, sortTable } from "./table";
import { createPageSearch } from "./search";

export { closeDialog, openDialog } from "./dialog";
export { alert, confirm } from "./confirm-dialog";
export type { AlertOptions, ConfirmOptions } from "./confirm-dialog";
export { closeDrawer, openDrawer } from "./drawer";
export { notify } from "./toast";
export type { ToastHandle, ToastOptions, ToastVariant } from "./toast";
export { selectTab } from "./tabs";
export { toggleAccordionItem } from "./accordion";
export { selectDropdownOption, toggleDropdown } from "./dropdown";
export { filterTable, sortTable } from "./table";
export { createPageSearch } from "./search";
export type { PageSearch } from "./search";

/**
 * Forma agrupada (Fase 16-adjacent fix), igual que `platform`/`stripe`:
 * `nexa-activation` detecta el uso de un identificador vía `<nombre>.`
 * en el código fuente de un handler (`ui.openDialog(...)`), no llamadas
 * bare (`openDialog(...)`) — así que la única forma realmente utilizable
 * desde una página es a través de este objeto.
 */
export const ui = {
    openDialog,
    closeDialog,
    confirm,
    alert,
    openDrawer,
    closeDrawer,
    notify,
    selectTab,
    toggleAccordionItem,
    toggleDropdown,
    selectDropdownOption,
    sortTable,
    filterTable,
    createPageSearch,
};
