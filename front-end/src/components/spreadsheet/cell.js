

export class Cell {


    /**
     * 
     * @param {number} rowId - row identifier [0..n] (where 0 - is header)
     * @param {number} colId - column identifier [0..n]
     * @param {string} data - data to display
     * @param {number} width - cell width in px
     * @param {number} height - cell height in px
     * @param {"left" | "right" | "center"} alignment - data alignment inside cell box
     */
    constructor( rowId, colId, data, width, height, alignment ) {
        this.rowId = rowId;
        this.colId = colId;
        this.data = data;
        this.width = width;
        this.height = height;
        this.alignment = alignment;

        let classList = (rowId === 0) ? "cell-header" : "cell-data";

        this.template = document.createElement("div");
        this.template.id = `${rowId}-${colId}`;
        this.template.style = `width: ${width}px; height: ${height}px; text-align: ${alignment}`;
        this.template.classList.add(classList);
        this.template.innerHTML = `<span class="extra-information">${data}</span>`;
    }

    /**
     * @returns {HTMLElement} - return cell HTML template
     */
    htmlTemplate() {
        return this.template;
    }

    
}