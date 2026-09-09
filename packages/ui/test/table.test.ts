import { describe, expect, it } from "vitest";
import { filterTable, sortTable } from "../src";

function setupTable(): { table: HTMLTableElement; th: HTMLTableCellElement } {
    document.body.innerHTML = `
        <table class="nx-table">
            <thead><tr><th data-sortable>Cliente</th><th data-sortable>Total</th></tr></thead>
            <tbody>
                <tr><td>Beta</td><td>30</td></tr>
                <tr><td>Alfa</td><td>10</td></tr>
                <tr><td>Gama</td><td>20</td></tr>
            </tbody>
        </table>
    `;
    return {
        table: document.querySelector(".nx-table") as HTMLTableElement,
        th: document.querySelector("th") as HTMLTableCellElement,
    };
}

function click(th: HTMLElement): void {
    const event = new MouseEvent("click", { bubbles: true });
    Object.defineProperty(event, "currentTarget", { value: th });
    sortTable(event);
}

function firstColumnValues(table: HTMLTableElement, columnIndex: number): string[] {
    return Array.from(table.querySelectorAll("tbody tr")).map((row) => row.children[columnIndex]?.textContent ?? "");
}

describe("sortTable", () => {
    it("ordena por texto ascendente al clickear el header por primera vez", () => {
        const { table, th } = setupTable();
        click(th);

        expect(firstColumnValues(table, 0)).toEqual(["Alfa", "Beta", "Gama"]);
        expect(th.getAttribute("aria-sort")).toBe("ascending");
    });

    it("un segundo click en el mismo header invierte el orden", () => {
        const { table, th } = setupTable();
        click(th);
        click(th);

        expect(firstColumnValues(table, 0)).toEqual(["Gama", "Beta", "Alfa"]);
        expect(th.getAttribute("aria-sort")).toBe("descending");
    });

    it("ordena numéricamente, no alfabéticamente, cuando la columna es de números", () => {
        const { table } = setupTable();
        const totalHeader = document.querySelectorAll("th")[1] as HTMLTableCellElement;
        click(totalHeader);

        expect(firstColumnValues(table, 1)).toEqual(["10", "20", "30"]);
    });
});

describe("filterTable", () => {
    it("oculta las filas cuyo texto no contiene la búsqueda", () => {
        const { table } = setupTable();
        filterTable(table, "alfa");

        const rows = Array.from(table.querySelectorAll("tbody tr")) as HTMLElement[];
        expect(rows.map((row) => row.hidden)).toEqual([true, false, true]);
    });

    it("una búsqueda vacía muestra todas las filas de nuevo", () => {
        const { table } = setupTable();
        filterTable(table, "alfa");
        filterTable(table, "");

        const rows = Array.from(table.querySelectorAll("tbody tr")) as HTMLElement[];
        expect(rows.every((row) => !row.hidden)).toBe(true);
    });

    it("no distingue mayúsculas/minúsculas", () => {
        const { table } = setupTable();
        filterTable(table, "BETA");

        const rows = Array.from(table.querySelectorAll("tbody tr")) as HTMLElement[];
        expect(rows.map((row) => row.hidden)).toEqual([false, true, true]);
    });
});
