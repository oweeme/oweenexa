/**
 * Comportamiento client-side de `.nx-table`: ordenar por columna y
 * filtrar filas por texto — sobre datos que el servidor ya renderizó
 * (vía `data.*`/`<For>`), sin volver a pedir nada al backend. Ninguna
 * isla: son funciones normales llamadas desde `onClick`/`onInput`
 * (Fase 57).
 */

/**
 * Ordena las filas de `<tbody>` por la columna del `<th>` clickeado.
 * Alterna ascendente/descendente en el mismo `<th>`; compara como
 * número si ambas celdas lo son, si no como texto.
 *
 * Marcado esperado: `<th onClick={sortByColumn} data-sortable>Precio</th>`
 * (con `function sortByColumn(event) { ui.sortTable(event); }` en la página).
 */
export function sortTable(event: Event): void {
    const th = event.currentTarget as HTMLTableCellElement | null;
    const table = th?.closest<HTMLTableElement>(".nx-table");
    const headerRow = th?.parentElement;
    if (!th || !table || !headerRow) return;

    const columnIndex = Array.from(headerRow.children).indexOf(th);
    if (columnIndex === -1) return;

    const nextDir = th.getAttribute("aria-sort") === "ascending" ? "descending" : "ascending";
    for (const header of Array.from(headerRow.children)) {
        header.removeAttribute("aria-sort");
    }
    th.setAttribute("aria-sort", nextDir);

    const tbody = table.querySelector("tbody");
    if (!tbody) return;

    const direction = nextDir === "ascending" ? 1 : -1;
    const rows = Array.from(tbody.querySelectorAll("tr"));

    rows.sort((rowA, rowB) => {
        const cellA = rowA.children[columnIndex]?.textContent?.trim() ?? "";
        const cellB = rowB.children[columnIndex]?.textContent?.trim() ?? "";
        const numA = Number(cellA);
        const numB = Number(cellB);
        if (cellA !== "" && cellB !== "" && !Number.isNaN(numA) && !Number.isNaN(numB)) {
            return (numA - numB) * direction;
        }
        return cellA.localeCompare(cellB) * direction;
    });

    for (const row of rows) {
        tbody.appendChild(row);
    }
}

/**
 * Oculta las filas de `<tbody>` cuyo texto no contenga `query`
 * (insensible a mayúsculas). Pensada para un `<input onInput={...}>`
 * separado que le pase su propio valor.
 *
 * Marcado esperado:
 * ```
 * function filterOrders(event) { ui.filterTable(document.getElementById("orders"), event.target.value); }
 * <input type="search" onInput={filterOrders} />
 * <table id="orders" class="nx-table">...</table>
 * ```
 */
export function filterTable(table: Element, query: string): void {
    const tbody = table.querySelector("tbody");
    if (!tbody) return;

    const q = query.trim().toLowerCase();
    for (const row of Array.from(tbody.querySelectorAll<HTMLElement>("tr"))) {
        const text = (row.textContent ?? "").toLowerCase();
        row.hidden = q.length > 0 && !text.includes(q);
    }
}
